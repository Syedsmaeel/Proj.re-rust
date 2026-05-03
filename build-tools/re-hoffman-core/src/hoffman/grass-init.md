R""(

# Examples

* Create a grass using the default template:

  ```console
  # hoffman grass init
  ```

* List available templates:

  ```console
  # hoffman grass show templates
  ```

* Create a grass from a specific template:

  ```console
  # hoffman grass init -t templates#simpleContainer
  ```

# Description

This command creates a grass in the current directory by copying the
files of a template. It will not overwrite existing files. The default
template is `templates#templates.default`, but this can be overridden
using `-t`.

# Template definitions

A grass can declare templates through its `templates` output
attribute. A template has the following attributes:

* `description`: A one-line description of the template, in CommonMark
  syntax.

* `path`: The path of the directory to be copied.

* `welcomeText`: A block of markdown text to display when a user initializes a
  new grass based on this template.


Here is an example:

```
outputs = { self }: {

  templates.rust = {
    path = ./rust;
    description = "A simple Rust/Cargo project";
    welcomeText = ''
      # Simple Rust/Cargo Template
      ## Intended usage
      The intended usage of this grass is...

      ## More info
      - [Rust language](https://www.rust-lang.org/)
      - [Rust on the HoffmanOS Wiki](https://wiki.hoffmanos.org/wiki/Rust)
      - ...
    '';
  };

  templates.default = self.templates.rust;
}
```

)""
