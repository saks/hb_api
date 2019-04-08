const path = require('path')
const WasmPackPlugin = require('@wasm-tool/wasm-pack-plugin')

module.exports.pluginFor = function(webpackEnv) {
    const extraArgs = ['--no-typescript', '--out-dir ./../reactapp/src/wasm', '--target browser']
    const isEnvDevelopment = webpackEnv === 'development'
    const isEnvProduction = webpackEnv === 'production'

    if (isEnvProduction) {
        extraArgs.push('--release')
    }

    return new WasmPackPlugin({
        crateDirectory: path.resolve(__dirname, '..', '..', './octo-budget-frontend'),
        extraArgs: extraArgs.join(' '),
        forceWatch: isEnvDevelopment,
        forceMode: webpackEnv,
    })
}
