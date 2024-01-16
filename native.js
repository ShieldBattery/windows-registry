// TODO(tec27): Pick between 32 and 64-bit, as well as whether to use a locally built binary?
let native
try {
  native = require('./prebuilds/x64.node')
} catch (e) {
  native = require('./native/index.node')
}

module.exports = native
