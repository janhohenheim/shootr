import typescript from 'rollup-plugin-typescript'
import babel from 'rollup-plugin-babel'
// import uglify from 'rollup-plugin-uglify-es'
import resolve from 'rollup-plugin-node-resolve'
import commonjs from 'rollup-plugin-commonjs'
import builtins from 'rollup-plugin-node-builtins'

export default {
  entry: './src/index.ts',
  dest: './public/index.js',
  format: 'iife',
  sourceMap: true,
  plugins: [
    typescript({
      typescript: require('typescript')
    }),
    babel({
      exclude: 'node_modules/**',
    }),
    resolve({
      // use "module" field for ES6 module if possible
      module: true, // Default: true

      // use "jsnext:main" if possible
      // – see https://github.com/rollup/rollup/wiki/jsnext:main
      jsnext: true,  // Default: false

      // not all files you want to resolve are .js files
      extensions: ['.js', '.json', '.ts'],  // Default: ['.js']

      // whether to prefer built-in modules (e.g. `fs`, `path`) or
      // local ones with the same names
      preferBuiltins: false,  // Default: true

      // If true, inspect resolved files to check that they are
      // ES2015 modules
      modulesOnly: false, // Default: false
    }),
    commonjs(),
    builtins(),
    // uglify(),
  ]
}