R""(

# Examples

* Upgrade Hoffman to the stable version declared in Hoffmanpkgs:

  ```console
  # hoffman upgrade-hoffman
  ```

* Upgrade Hoffman in a specific profile:

  ```console
  # hoffman upgrade-hoffman --profile ~alice/.local/state/hoffman/profiles/profile
  ```

# Description

This command upgrades Hoffman to the stable version.

By default, the latest stable version is defined by Hoffmanpkgs, in
[hoffman-fallback-paths.hoffman](https://github.com/HoffmanOS/hoffmanpkgs/raw/master/hoffmanos/modules/installer/tools/hoffman-fallback-paths.hoffman)
and updated manually. It may not always be the latest tagged release.

By default, it locates the directory containing the `hoffman` binary in the `$PATH`
environment variable. If that directory is a Hoffman profile, it will
upgrade the `hoffman` package in that profile to the latest stable binary
release.

You cannot use this command to upgrade Hoffman in the system profile of a
HoffmanOS system (that is, if `hoffman` is found in `/run/current-system`).

)""
