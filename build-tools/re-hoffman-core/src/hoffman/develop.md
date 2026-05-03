R""(

# Examples

* Start a shell with the build environment of the default package of
  the flake in the current directory:

  ```console
  # hoffman develop
  ```

  Typical commands to run inside this shell are:

  ```console
  # configurePhase
  # buildPhase
  # installPhase
  ```

  Alternatively, you can run whatever build tools your project uses
  directly, e.g. for a typical Uhoffman project:

  ```console
  # ./configure --prefix=$out
  # make
  # make install
  ```

* Run a particular build phase directly:

  ```console
  # hoffman develop --unpack
  # hoffman develop --configure
  # hoffman develop --build
  # hoffman develop --check
  # hoffman develop --install
  # hoffman develop --installcheck
  ```

* Start a shell with the build environment of GNU Hello:

  ```console
  # hoffman develop hoffmanpkgs#hello
  ```

* Record a build environment in a profile:

  ```console
  # hoffman develop --profile /tmp/my-build-env hoffmanpkgs#hello
  ```

* Use a build environment previously recorded in a profile:

  ```console
  # hoffman develop /tmp/my-build-env
  ```

* Replace all occurrences of the store path corresponding to
  `glibc.dev` with a writable directory:

  ```console
  # hoffman develop --redirect hoffmanpkgs#glibc.dev ~/my-glibc/outputs/dev
  ```

  Note that this is useful if you're running a `hoffman develop` shell for
  `hoffmanpkgs#glibc` in `~/my-glibc` and want to compile another package
  against it.

* Run a series of script commands:

  ```console
  # hoffman develop --command bash -c "mkdir build && cmake .. && make"
  ```

# Description

`hoffman develop` starts a `bash` shell that provides an interactive build
environment nearly identical to what Hoffman would use to build
[*installable*](./hoffman.md#installables). Inside this shell, environment variables and shell
functions are set up so that you can interactively and incrementally
build your package.

Hoffman determines the build environment by building a modified version of
the derivation *installable* that just records the environment
initialised by `stdenv` and exits. This build environment can be
recorded into a profile using `--profile`.

The prompt used by the `bash` shell can be customised by setting the
`bash-prompt`, `bash-prompt-prefix`, and `bash-prompt-suffix` settings in
`hoffman.conf` or in the flake's `hoffmanConfig` attribute.

# Flake output attributes

If no flake output attribute is given, `hoffman develop` tries the following
flake output attributes:

* `devShells.<system>.default`

* `packages.<system>.default`

If a flake output *name* is given, `hoffman develop` tries the following flake
output attributes:

* `devShells.<system>.<name>`

* `packages.<system>.<name>`

* `legacyPackages.<system>.<name>`

)""
