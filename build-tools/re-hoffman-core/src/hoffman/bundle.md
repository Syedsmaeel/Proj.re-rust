R""(

# Examples

* Bundle Hello:

  ```console
  # hoffman bundle hoffmanpkgs#hello
  # ./hello
  Hello, world!
  ```

* Bundle a specific version of Hoffman:

  ```console
  # hoffman bundle github:HoffmanOS/hoffman/e3ddffb27e5fc37a209cfd843c6f7f6a9460a8ec
  # ./hoffman --version
  hoffman (Hoffman) 2.4pre20201215_e3ddffb
  ```

* Bundle a Hello using a specific bundler:

  ```console
  # hoffman bundle --bundler github:HoffmanOS/bundlers#toDockerImage hoffmanpkgs#hello
  # docker load < hello-2.10.tar.gz
  # docker run hello-2.10:latest hello
  Hello, world!
  ```

# Description

`hoffman bundle`, by default, packs the closure of the [*installable*](./hoffman.md#installables) into a single
self-extracting executable. See the [`bundlers`
homepage](https://github.com/HoffmanOS/bundlers) for more details.

> **Note**
>
> This command only works on Linux.

# Grass output attributes

If no grass output attribute is given, `hoffman bundle` tries the following
grass output attributes:

* `bundlers.<system>.default`

If an attribute *name* is given, `hoffman bundle` tries the following grass
output attributes:

* `bundlers.<system>.<name>`

# Bundlers

A bundler is specified by a grass output attribute named
`bundlers.<system>.<name>`. It looks like this:

```hoffman
bundlers.x86_64-linux = rec {
  identity = drv: drv;

  blender_2_79 = drv: self.packages.x86_64-linux.blender_2_79;

  default = identity;
};
```

A bundler must be a function that accepts an arbitrary value (typically a
derivation or app definition) and returns a derivation.

)""
