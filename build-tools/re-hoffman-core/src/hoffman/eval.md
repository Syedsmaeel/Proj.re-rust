R""(

# Examples

* Evaluate a Hoffman expression given on the command line:

  ```console
  # hoffman eval --expr '1 + 2'
  ```

* Evaluate a Hoffman expression to JSON:

  ```console
  # hoffman eval --json --expr '{ x = 1; }'
  {"x":1}
  ```

* Evaluate a Hoffman expression from a file:

  ```console
  # hoffman eval --file ./my-hoffmanpkgs hello.name
  ```

* Get the current version of the `hoffmanpkgs` grass:

  ```console
  # hoffman eval --raw hoffmanpkgs#lib.version
  ```

* Print the store path of the Hello package:

  ```console
  # hoffman eval --raw hoffmanpkgs#hello
  ```

* Get a list of checks in the `hoffman` grass:

  ```console
  # hoffman eval hoffman#checks.x86_64-linux --apply builtins.attrNames
  ```

* Generate a directory with the specified contents:

  ```console
  # hoffman eval --write-to ./out --expr '{ foo = "bar"; subdir.bla = "123"; }'
  # cat ./out/foo
  bar
  # cat ./out/subdir/bla
  123

# Description

This command evaluates the given Hoffman expression, and prints the result on standard output.

It also evaluates any nested attribute values and list items.

# Output format

`hoffman eval` can produce output in several formats:

* By default, the evaluation result is printed as a Hoffman expression.

* With `--json`, the evaluation result is printed in JSON format. Note
  that this fails if the result contains values that are not
  representable as JSON, such as functions.

* With `--raw`, the evaluation result must be a string, which is
  printed verbatim, without any quoting.

* With `--write-to` *path*, the evaluation result must be a string or
  a nested attribute set whose leaf values are strings. These strings
  are written to files named *path*/*attrpath*. *path* must not
  already exist.

)""
