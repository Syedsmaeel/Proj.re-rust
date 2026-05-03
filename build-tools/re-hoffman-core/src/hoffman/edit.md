R""(

# Examples

* Open the Hoffman expression of the GNU Hello package:

  ```console
  # hoffman edit hoffmanpkgs#hello
  ```

* Get the filename and line number used by `hoffman edit`:

  ```console
  # hoffman eval --raw hoffmanpkgs#hello.meta.position
  /hoffman/store/fvafw0gvwayzdan642wrv84pzm5bgpmy-source/pkgs/applications/misc/hello/default.hoffman:15
  ```

# Description

This command opens the Hoffman expression of a derivation in an
editor. The filename and line number of the derivation are taken from
its `meta.position` attribute. Hoffmanpkgs' `stdenv.mkDerivation` sets
this attribute to the location of the definition of the
`meta.description`, `version` or `name` derivation attributes.

The editor to invoke is specified by the `EDITOR` environment
variable. It defaults to `cat`. If the editor is `emacs`, `nano`,
`vim` or `kak`, it is passed the line number of the derivation using
the argument `+<lineno>`.

)""
