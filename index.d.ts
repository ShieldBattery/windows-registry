export class WindowsRegistry {
  /**
   * Constructs a new `WindowsRegistry` instance. This will start a background thread to perform
   * operations that will live as long as this instance does.
   */
  constructor()
  /**
   * Closes the registry, ending its background thread immediately. This is not necessary to call,
   * as the background thread will be automatically ended upon garbage collection, but can be used
   * if you would like to exit the process immediately.
   */
  close(): void

  read(hive: Hkey, key: string, name: string): Promise<RegistryDataTypeJs[RegistryDataType]>
  write<R extends RegistryDataType>(
    hive: Hkey,
    key: string,
    name: string,
    type: R,
    value: RegistryDataTypeJs[R],
  ): Promise<void>
}

export const HKCU = 'HKEY_CURRENT_USER'
export const HKLM = 'HKEY_LOCAL_MACHINE'
export const HKCR = 'HKEY_CLASSES_ROOT'
export const HKU = 'HKEY_USERS'
export const HKCC = 'HKEY_CURRENT_CONFIG'
export type Hkey = typeof HKCU | typeof HKLM | typeof HKCR | typeof HKU | typeof HKCC

export const REG_NONE = 'REG_NONE'
export const REG_SZ = 'REG_SZ'
export const REG_MULTI_SZ = 'REG_MULTI_SZ'
export const REG_EXPAND_SZ = 'REG_EXPAND_SZ'
export const REG_DWORD = 'REG_DWORD'
export const REG_QWORD = 'REG_QWORD'
export const REG_BINARY = 'REG_BINARY'
export type RegistryDataType =
  | typeof REG_NONE
  | typeof REG_SZ
  | typeof REG_MULTI_SZ
  | typeof REG_EXPAND_SZ
  | typeof REG_DWORD
  | typeof REG_QWORD
  | typeof REG_BINARY

interface RegistryDataTypeJs extends Record<RegistryDataType, any> {
  [REG_NONE]: undefined | null
  [REG_SZ]: string
  [REG_MULTI_SZ]: string[]
  [REG_EXPAND_SZ]: string
  [REG_DWORD]: number
  [REG_QWORD]: number
  [REG_BINARY]: Uint8Array
}
