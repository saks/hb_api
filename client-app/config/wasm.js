const path = require('path');
const WasmPackPlugin = require('@wasm-tool/wasm-pack-plugin');

module.exports.pluginFor = function(webpackEnv, target) {
    const extraArgs = ['--no-typescript', `--target ${target}`];
    const isEnvDevelopment = webpackEnv === 'development';
    const isEnvProduction = webpackEnv === 'production';
    const outDir = 'nodejs' === target ? 'wasm-test' : 'wasm';

    return new WasmPackPlugin({
        crateDirectory: path.resolve(__dirname, '..', '..', './octo-budget-frontend'),
        extraArgs: extraArgs.join(' '),
        forceWatch: isEnvDevelopment,
        forceMode: webpackEnv,
        outDir: `./../client-app/src/${outDir}`,
    });
};
