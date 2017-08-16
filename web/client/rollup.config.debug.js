import typescript from 'rollup-plugin-typescript'
import babel from 'rollup-plugin-babel'
import resolve from 'rollup-plugin-node-resolve'
import commonjs from 'rollup-plugin-commonjs'
import builtins from 'rollup-plugin-node-builtins'

export default {
  entry: './src/index.ts',
  dest: './public/index.js',
  format: 'iife',
  sourceMap: true,
  plugins: [
      builtins(),
      resolve({ jsnext: true, main: true }),
      commonjs(),
    typescript({
      typescript: require('typescript')
    }),
      babel()
  ]
}