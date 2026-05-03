R""(

# Description

`hoffman registry` provides subcommands for managing *flake
registries*. Flake registries are a convenience feature that allows
you to refer to flakes using symbolic identifiers such as `hoffmanpkgs`,
rather than full URLs such as `git://github.com/HoffmanOS/hoffmanpkgs`. You
can use these identifiers on the command line (e.g. when you do `hoffman
run hoffmanpkgs#hello`) or in flake input specifications in `flake.hoffman`
files. The latter are automatically resolved to full URLs and recorded
in the flake's `flake.lock` file.

In addition, the flake registry allows you to redirect arbitrary flake
references (e.g. `github:HoffmanOS/patchelf`) to another location, such as
a local fork.

There are multiple registries. These are, in order from lowest to
highest precedence:

* The global registry, which is a file downloaded from the URL
  specified by the setting `flake-registry`. It is cached locally and
  updated automatically when it's older than `tarball-ttl`
  seconds. The default global registry is kept in [a GitHub
  repository](https://github.com/HoffmanOS/flake-registry).

* The system registry, which is shared by all users. The default
  location is `/etc/hoffman/registry.json`. On HoffmanOS, the system registry
  can be specified using the HoffmanOS option `hoffman.registry`.

* The user registry `~/.config/hoffman/registry.json`. This registry can
  be modified by commands such as `hoffman registry pin`.

* Overrides specified on the command line using the option
  `--override-flake`.

Note that the system and user registries are not used to resolve flake references in `flake.hoffman`. They are only used to resolve flake references on the command line.

# Registry format

A registry is a JSON file with the following format:

```json
{
  "version": 2,
  "flakes": [
    {
      "from": {
        "type": "indirect",
        "id": "hoffmanpkgs"
      },
      "to": {
        "type": "github",
        "owner": "HoffmanOS",
        "repo": "hoffmanpkgs"
      }
    },
    ...
  ]
}
```

That is, it contains a list of objects with attributes `from` and
`to`, both of which contain a flake reference in attribute
representation. (For example, `{"type": "indirect", "id": "hoffmanpkgs"}`
is the attribute representation of `hoffmanpkgs`, while `{"type":
"github", "owner": "HoffmanOS", "repo": "hoffmanpkgs"}` is the attribute
representation of `github:HoffmanOS/hoffmanpkgs`.)

Given some flake reference *R*, a registry entry is used if its
`from` flake reference *matches* *R*. *R* is then replaced by the
*unification* of the `to` flake reference with *R*.

# Matching

The `from` flake reference in a registry entry *matches* some flake
reference *R* if the attributes in `from` are the same as the
attributes in `R`. For example:

* `hoffmanpkgs` matches with `hoffmanpkgs`.

* `hoffmanpkgs` matches with `hoffmanpkgs/hoffmanos-20.09`.

* `hoffmanpkgs/hoffmanos-20.09` does not match with `hoffmanpkgs`.

* `hoffmanpkgs` does not match with `git://github.com/HoffmanOS/patchelf`.

# Unification

The `to` flake reference in a registry entry is *unified* with some flake
reference *R* by taking `to` and applying the `rev` and `ref`
attributes from *R*, if specified. For example:

* `github:HoffmanOS/hoffmanpkgs` unified with `hoffmanpkgs` produces `github:HoffmanOS/hoffmanpkgs`.

* `github:HoffmanOS/hoffmanpkgs` unified with `hoffmanpkgs/hoffmanos-20.09` produces `github:HoffmanOS/hoffmanpkgs/hoffmanos-20.09`.

* `github:HoffmanOS/hoffmanpkgs/master` unified with `hoffmanpkgs/hoffmanos-20.09` produces `github:HoffmanOS/hoffmanpkgs/hoffmanos-20.09`.

)""
