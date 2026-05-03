# Options

The following options are allowed for all `hoffman-env` operations, but may not always have an effect.

- `--file` / `-f` *path*

  Specifies the Hoffman expression (designated below as the *active Hoffman
  expression*) used by the `--install`, `--upgrade`, and `--query
  --available` operations to obtain derivations. The default is
  `~/.hoffman-defexpr`.

  If the argument starts with `http://` or `https://`, it is
  interpreted as the URL of a tarball that will be downloaded and
  unpacked to a temporary location. The tarball must include a single
  top-level directory containing at least a file named `default.hoffman`.

- `--profile` / `-p` *path*

  Specifies the profile to be used by those operations that operate on
  a profile (designated below as the *active profile*). A profile is a
  sequence of user environments called *generations*, one of which is
  the *current generation*.

- `--dry-run`

  For the `--install`, `--upgrade`, `--uninstall`,
  `--switch-generation`, `--delete-generations` and `--rollback`
  operations, this flag will cause `hoffman-env` to print what *would* be
  done if this flag had not been specified, without actually doing it.

  `--dry-run` also prints out which paths will be
  [substituted](@docroot@/glossary.md) (i.e., downloaded) and which paths
  will be built from source (because no substitute is available).

- `--system-filter` *system*

  By default, operations such as `--query --available` show derivations matching any platform. This option
  allows you to use derivations for the specified platform *system*.
