const path = require('path');
const TerserPlugin = require('terser-webpack-plugin');
const production = process.env.NODE_ENV === 'production' || false;

module.exports = {
    entry: ['./src/coinbacked.ts'], 
    module: {
        rules: [
            {
                test: /\.ts?$/,
                use: 'ts-loader',
                exclude: /node_modules/,
            },
        ],
    },
    resolve: {
        extensions: ['.ts', '.js']
    },
    mode: 'production',
    output: {
        filename: production ? 'coinbacked.min.js' : 'coinbacked.js',
        path: path.resolve(__dirname, 'dist'),
        globalObject: 'this',
        library: "coinbackedWeb3",
        libraryExport: "default",
        libraryTarget: "umd",
        
    },
    optimization: {
        minimize: production,
        minimizer: [
            new TerserPlugin({})
        ]
    },
    externals:
    {
        "@solana/web3.js": "solanaWeb3"
    } 
}