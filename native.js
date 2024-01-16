// TODO(tec27): Pick between 32 and 64-bit, as well as whether to use a locally built binary?
let native
try {
  native = require('./prebuilds/x64.node')
} catch (e) {
  // Workaround to fix webpack's build warnings: 'the request of a dependency is an expression'
  const runtimeRequire =
    typeof __webpack_require__ === 'function' ? __non_webpack_require__ : require
  native = runtimeRequire('./native/index.node')
}

module.exports = native
