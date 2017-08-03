import typescript from 'rollup-plugin-typescript'
import babel from 'rollup-plugin-babel'

export default {
  entry: './src/index.ts',
  dest: './public/index.js',
  format: 'iife',
  sourceMap: true,
  plugins: [
    typescript({
      typescript: require('typescript')
    }),
    babel()
  ]
}