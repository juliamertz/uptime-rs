const { nodeResolve } = require("@rollup/plugin-node-resolve");
const { uglify } = require("rollup-plugin-uglify");

exports.default = {
  input: "src/index.mjs",
  output: {
    dir: "output",
    format: "es",
  },
  plugins: [
    nodeResolve(), //
    uglify(),
  ],
};
