const test = require('node:test')
const assert = require('node:assert/strict')

const {
  WindowsRegistry,
  HKCU,
  REG_SZ,
  REG_DWORD,
  REG_QWORD,
  REG_NONE,
  REG_MULTI_SZ,
  REG_EXPAND_SZ,
  REG_BINARY,
} = require('../index.js')

const TEST_KEY = 'SOFTWARE\\windows-registry\\test'

test('roundtrips string values', async () => {
  const registry = new WindowsRegistry()
  try {
    const randomValue = String(Math.round(Math.random() * 9999999))
    await registry.write(HKCU, TEST_KEY, 'RoundTripString', REG_SZ, randomValue)
    // NOTE(tec27): We write a different value to the same key to ensure that creating the key does
    // not clear its values
    await registry.write(HKCU, TEST_KEY, 'RoundTripString2', REG_SZ, randomValue)

    const readValue = await registry.read(
      HKCU,
      'SOFTWARE\\windows-registry\\test',
      'RoundTripString',
    )
    assert.equal(readValue, randomValue)
  } finally {
    registry.close()
  }
})

test('roundtrips dword values', async () => {
  const registry = new WindowsRegistry()
  try {
    const randomValue = Math.round(Math.random() * 9999999)
    await registry.write(HKCU, TEST_KEY, 'RoundTripDword', REG_DWORD, randomValue)

    const readValue = await registry.read(HKCU, TEST_KEY, 'RoundTripDword')
    assert.equal(readValue, randomValue)
  } finally {
    registry.close()
  }
})

test('roundtrips qword values', async () => {
  const registry = new WindowsRegistry()
  try {
    const randomValue = Math.round(Math.random() * 999999999)
    await registry.write(HKCU, TEST_KEY, 'RoundTripQword', REG_QWORD, randomValue)

    const readValue = await registry.read(HKCU, TEST_KEY, 'RoundTripQword')
    assert.equal(readValue, randomValue)
  } finally {
    registry.close()
  }
})

test('roundtrips none values', async () => {
  const registry = new WindowsRegistry()
  try {
    await registry.write(HKCU, TEST_KEY, 'RoundTripNone', REG_NONE)
    const readValue = await registry.read(HKCU, TEST_KEY, 'RoundTripNone')
    assert.equal(readValue, null)
  } finally {
    registry.close()
  }
})

test('roundtrips binary values', async () => {
  const registry = new WindowsRegistry()
  try {
    const randomValue = Buffer.from([
      Math.round(Math.random() * 255),
      Math.round(Math.random() * 255),
    ])
    await registry.write(HKCU, TEST_KEY, 'RoundTripBinary', REG_BINARY, randomValue)

    const readValue = await registry.read(HKCU, TEST_KEY, 'RoundTripBinary')
    assert.deepEqual(readValue, randomValue)
  } finally {
    registry.close()
  }
})

test('roundtrips multi-string values', async () => {
  const registry = new WindowsRegistry()
  try {
    const randomValue = [
      String(Math.round(Math.random() * 9999999)),
      String(Math.round(Math.random() * 9999999)),
    ]
    await registry.write(HKCU, TEST_KEY, 'RoundTripMultiString', REG_MULTI_SZ, randomValue)

    const readValue = await registry.read(HKCU, TEST_KEY, 'RoundTripMultiString')
    assert.deepEqual(readValue, randomValue)
  } finally {
    registry.close()
  }
})

test('roundtrips expand-string values', async () => {
  const registry = new WindowsRegistry()
  try {
    const randomValue = String(Math.round(Math.random() * 9999999))
    await registry.write(HKCU, TEST_KEY, 'RoundTripExpandString', REG_EXPAND_SZ, randomValue)

    const readValue = await registry.read(HKCU, TEST_KEY, 'RoundTripExpandString')
    assert.deepEqual(readValue, randomValue)
  } finally {
    registry.close()
  }
})

test('rejects after close', async () => {
  const registry = new WindowsRegistry()
  registry.close()

  await assert.rejects(() => registry.read(HKCU, TEST_KEY, 'SomeValue'), /closed/)
})
