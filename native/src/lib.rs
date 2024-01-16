use std::sync::mpsc;
use std::thread;

use neon::{
    prelude::*,
    types::{buffer::TypedArray, Deferred},
};
use registry::{Data, Hive, Security};
use utfx::U16CString;

type RegistryCallback = Box<dyn FnOnce(/*&mut Connection,*/ &Channel, Deferred) + Send>;

struct RegistryThread {
    tx: mpsc::Sender<RegistryMessage>,
}

enum RegistryMessage {
    Callback(Deferred, RegistryCallback),
    Close,
}

impl Finalize for RegistryThread {}

impl RegistryThread {
    fn new<'a, C>(cx: &mut C) -> anyhow::Result<RegistryThread>
    where
        C: Context<'a>,
    {
        let (tx, rx) = mpsc::channel::<RegistryMessage>();

        let channel = cx.channel();

        thread::spawn(move || {
            while let Ok(message) = rx.recv() {
                match message {
                    RegistryMessage::Callback(deferred, f) => {
                        f(&channel, deferred);
                    }
                    RegistryMessage::Close => break,
                }
            }
        });

        Ok(Self { tx })
    }

    fn close(&self) -> Result<(), mpsc::SendError<RegistryMessage>> {
        self.tx.send(RegistryMessage::Close)
    }

    fn send(
        &self,
        deferred: Deferred,
        callback: impl FnOnce(&Channel, Deferred) + Send + 'static,
    ) -> Result<(), mpsc::SendError<RegistryMessage>> {
        self.tx
            .send(RegistryMessage::Callback(deferred, Box::new(callback)))
    }

    fn js_new(mut cx: FunctionContext) -> JsResult<JsBox<RegistryThread>> {
        let reg = RegistryThread::new(&mut cx).or_else(|err| cx.throw_error(err.to_string()))?;

        Ok(cx.boxed(reg))
    }

    /// Closes the associated RegistryThread immediately. This is not required, as the thread will
    /// close itself upon garbage collection, but this can be used to immediately end the thread to
    /// allow the process to shut down.
    fn js_close(mut cx: FunctionContext) -> JsResult<JsUndefined> {
        cx.this()
            .downcast_or_throw::<JsBox<RegistryThread>, _>(&mut cx)?
            .close()
            .or_else(|err| cx.throw_error(err.to_string()))?;

        Ok(cx.undefined())
    }

    /// Reads a value from the registry.
    /// Arguments are:
    ///     - `hive` (string): The hive to read from
    ///     - `key` (string): The key inside the hive to read
    ///     - `value` (string): The value inside the key to read
    fn js_read(mut cx: FunctionContext) -> JsResult<JsPromise> {
        let hive = cx.argument::<JsString>(0)?.value(&mut cx);
        let key = cx.argument::<JsString>(1)?.value(&mut cx);
        let value = cx.argument::<JsString>(2)?.value(&mut cx);

        let hive = hkey_str_to_hive(&hive).or_else(|e| cx.throw_error(e.to_string()))?;

        let reg = cx
            .this()
            .downcast_or_throw::<JsBox<RegistryThread>, _>(&mut cx)?;
        let (deferred, promise) = cx.promise();

        reg.send(deferred, move |channel, deferred| {
            let reg_key = match hive.open(key, Security::Read) {
                Ok(r) => r,
                Err(e) => {
                    match e {
                        registry::key::Error::NotFound(_, _) => {
                            deferred.settle_with(channel, move |mut cx| Ok(cx.undefined()));
                        }
                        _ => {
                            deferred.settle_with(channel, move |mut cx| {
                                cx.throw_error::<std::string::String, Handle<'_, JsUndefined>>(
                                    e.to_string(),
                                )
                            });
                        }
                    }
                    return;
                }
            };

            let data = reg_key.value(value);
            deferred.settle_with(channel, move |mut cx| match data {
                Ok(data) => match data {
                    Data::None => Ok(cx.null().upcast::<JsValue>()),
                    Data::String(s) => Ok(cx.string(s.to_string_lossy()).upcast()),
                    Data::MultiString(s) => {
                        let js_array = JsArray::new(&mut cx, s.len() as u32);
                        for (i, s) in s.iter().enumerate() {
                            let s = cx.string(s.to_string_lossy());
                            js_array.set(&mut cx, i as u32, s)?;
                        }
                        Ok(js_array.upcast())
                    }
                    Data::ExpandString(s) => Ok(cx.string(s.to_string_lossy()).upcast()),
                    Data::U32(n) => Ok(cx.number(n as f64).upcast()),
                    Data::U64(n) => Ok(cx.number(n as f64).upcast()),
                    Data::Binary(b) => {
                        let typed_array = JsBuffer::external(&mut cx, b);
                        Ok(typed_array.upcast())
                    }
                    t => cx.throw_error(format!("Unsupported registry type: {}", t)),
                },
                Err(e) => match e {
                    registry::value::Error::NotFound(_, _) => Ok(cx.undefined().upcast()),
                    _ => cx.throw_error::<std::string::String, Handle<'_, JsValue>>(e.to_string()),
                },
            });
        })
        .into_rejection(&mut cx)?;

        Ok(promise)
    }

    /// Writes a value from the registry.
    /// Arguments are:
    ///     - `hive` (string): The hive to read from
    ///     - `key` (string): The key inside the hive to read
    ///     - `value` (string): The value inside the key to read
    ///     - `type` (string): The type of `data`
    ///     - `data` (depends on `type`): The data be written to the value
    fn js_write(mut cx: FunctionContext) -> JsResult<JsPromise> {
        let hive = cx.argument::<JsString>(0)?.value(&mut cx);
        let key = cx.argument::<JsString>(1)?.value(&mut cx);
        let value = cx.argument::<JsString>(2)?.value(&mut cx);
        let type_name = cx.argument::<JsString>(3)?.value(&mut cx);

        let hive = hkey_str_to_hive(&hive).or_else(|e| cx.throw_error(e.to_string()))?;
        let data = match type_name.as_str() {
            "REG_NONE" => Ok(Data::None),
            "REG_SZ" => Ok(Data::String(
                cx.argument::<JsString>(4)?
                    .value(&mut cx)
                    .try_into()
                    .or_else(|e| cx.throw_error(format!("Invalid string: {}", e)))?,
            )),
            "REG_MULTI_SZ" => Ok(Data::MultiString(
                cx.argument::<JsArray>(4)?
                    .to_vec(&mut cx)?
                    .iter()
                    .map(|s| {
                        s.downcast_or_throw::<JsString, _>(&mut cx)?
                            .value(&mut cx)
                            .try_into()
                            .or_else(|e| {
                                cx.throw_error::<std::string::String, U16CString>(format!(
                                    "Invalid string: {}",
                                    e
                                ))
                            })
                    })
                    .collect::<Result<Vec<U16CString>, _>>()?,
            )),
            "REG_EXPAND_SZ" => Ok(Data::ExpandString(
                cx.argument::<JsString>(4)?
                    .value(&mut cx)
                    .try_into()
                    .or_else(|e| cx.throw_error(format!("Invalid expand string: {}", e)))?,
            )),
            "REG_DWORD" => Ok(Data::U32(cx.argument::<JsNumber>(4)?.value(&mut cx) as u32)),
            "REG_QWORD" => Ok(Data::U64(cx.argument::<JsNumber>(4)?.value(&mut cx) as u64)),
            "REG_BINARY" => Ok(Data::Binary(
                cx.argument::<JsTypedArray<u8>>(4)?.as_slice(&cx).to_vec(),
            )),
            _ => Err(cx.throw_error(format!("Invalid registry type: {}", type_name))?),
        }?;

        let reg = cx
            .this()
            .downcast_or_throw::<JsBox<RegistryThread>, _>(&mut cx)?;
        let (deferred, promise) = cx.promise();

        reg.send(deferred, move |channel, deferred| {
            let reg_key = match hive.create(key, Security::Write) {
                Ok(r) => r,
                Err(e) => {
                    deferred.settle_with(channel, move |mut cx| {
                        cx.throw_error::<std::string::String, Handle<'_, JsUndefined>>(
                            e.to_string(),
                        )
                    });
                    return;
                }
            };

            match reg_key.set_value(value, &data) {
                Ok(_) => {
                    deferred.settle_with(channel, move |mut cx| Ok(cx.undefined()));
                }
                Err(e) => {
                    deferred.settle_with(channel, move |mut cx| {
                        cx.throw_error::<std::string::String, Handle<'_, JsUndefined>>(
                            e.to_string(),
                        )
                    });
                }
            }
        })
        .into_rejection(&mut cx)?;

        Ok(promise)
    }
}
trait SendResultExt {
    // Sending a closure to execute may fail if the channel has been closed.
    // This method converts the failure into a promise rejection.
    fn into_rejection<'a, C: Context<'a>>(self, cx: &mut C) -> NeonResult<()>;
}

impl SendResultExt for Result<(), mpsc::SendError<RegistryMessage>> {
    fn into_rejection<'a, C: Context<'a>>(self, cx: &mut C) -> NeonResult<()> {
        self.or_else(|err| {
            let msg = err.to_string();

            match err.0 {
                RegistryMessage::Callback(deferred, _) => {
                    let err = cx.error(msg)?;
                    deferred.reject(cx, err);
                    Ok(())
                }
                RegistryMessage::Close => cx.throw_error("Expected RegistryMessage::Callback"),
            }
        })
    }
}

fn hkey_str_to_hive(hkey: &str) -> anyhow::Result<Hive> {
    match hkey {
        "HKEY_CLASSES_ROOT" => Ok(Hive::ClassesRoot),
        "HKEY_CURRENT_CONFIG" => Ok(Hive::CurrentConfig),
        "HKEY_CURRENT_USER" => Ok(Hive::CurrentUser),
        "HKEY_LOCAL_MACHINE" => Ok(Hive::LocalMachine),
        "HKEY_USERS" => Ok(Hive::Users),
        _ => Err(anyhow::anyhow!("Invalid hive: {}", hkey)),
    }
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("registryNew", RegistryThread::js_new)?;
    cx.export_function("registryClose", RegistryThread::js_close)?;
    cx.export_function("registryRead", RegistryThread::js_read)?;
    cx.export_function("registryWrite", RegistryThread::js_write)?;
    Ok(())
}
