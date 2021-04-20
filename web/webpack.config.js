const path = require('path');
const HtmlWebpackPlugin = require('html-webpack-plugin');
const MiniCssExtractPlugin = require('mini-css-extract-plugin');
const CopyWebpackPlugin = require('copy-webpack-plugin');

const IS_PRODUCTION = process.env.NODE_ENV === 'production';
const SRC_PATH = path.resolve(__dirname, 'src');
const DIST_PATH = path.resolve(__dirname, 'dist');

module.exports = {
  mode: IS_PRODUCTION ? 'production' : 'development',

  entry: path.resolve(__dirname, 'src/js/index.js'),

  output: {
    path: DIST_PATH,
    filename: 'bundle.[contenthash].js',
  },

  module: {
    rules: [
      {
        test: /\.css$/,
        use: [MiniCssExtractPlugin.loader, 'css-loader', 'postcss-loader'],
      },
    ],
  },

  plugins: [
    new HtmlWebpackPlugin({
      template: path.join(SRC_PATH, 'index.html'),
      minify: true,
    }),
    new MiniCssExtractPlugin({
      filename: 'style.[contenthash].css',
    }),
    new CopyWebpackPlugin({
      patterns: [
        {
          from: '**/*',
          to: DIST_PATH,
          context: 'src',
          globOptions: {
            ignore: ['**/*.js', '**/*.css', '**/*.html'],
          },
        },
      ],
    }),
  ],
};
