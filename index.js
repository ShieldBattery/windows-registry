const { registryNew, registryClose, registryRead, registryWrite } = require('./native.js')

class WindowsRegistry {
  /**
   * Constructs a new `WindowsRegistry` instance. This will start a background thread to perform
   * operations that will live as long as this instance does.
   */
  constructor() {
    this.registry = registryNew()
  }

  /**
   * Closes the registry, ending its background thread immediately. This is not necessary to call,
   * as the background thread will be automatically ended upon garbage collection, but can be used
   * if you would like to exit the process immediately.
   */
  close() {
    registryClose.call(this.registry)
  }

  read(hive, key, value) {
    return registryRead.call(this.registry, hive, key, value)
  }

  write(hive, key, value, type, data) {
    return registryWrite.call(this.registry, hive, key, value, type, data)
  }
}

module.exports = {
  WindowsRegistry,

  HKCU: 'HKEY_CURRENT_USER',
  HKLM: 'HKEY_LOCAL_MACHINE',
  HKCR: 'HKEY_CLASSES_ROOT',
  HKU: 'HKEY_USERS',
  HKCC: 'HKEY_CURRENT_CONFIG',

  REG_NONE: 'REG_NONE',
  REG_SZ: 'REG_SZ',
  REG_MULTI_SZ: 'REG_MULTI_SZ',
  REG_EXPAND_SZ: 'REG_EXPAND_SZ',
  REG_DWORD: 'REG_DWORD',
  REG_QWORD: 'REG_QWORD',
  REG_BINARY: 'REG_BINARY',

  DEFAULT_VALUE: '',
}
