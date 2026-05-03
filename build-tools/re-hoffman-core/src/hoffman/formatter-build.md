R""(

# Description

`hoffman formatter build` builds the formatter specified in the flake.

Similar to [`hoffman build`](@docroot@/command-ref/new-cli/hoffman3-build.md),
unless `--no-link` is specified, after a successful
build, it creates a symlink to the store path of the formatter. This symlink is
named `./result` by default; this can be overridden using the
`--out-link` option.

It always prints the command to standard output.

# Examples

* Build the formatter:

  ```console
  # hoffman formatter build
  /hoffman/store/cb9w44vkhk2x4adfxwgdkkf5gjmm856j-treefmt/bin/treefmt
  ```
)""
