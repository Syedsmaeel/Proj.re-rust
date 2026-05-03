R""(

# Examples

* Resolve the `hoffmanpkgs` and `blender-bin` flakerefs:

  ```console
  # hoffman registry resolve hoffmanpkgs blender-bin
  github:HoffmanOS/hoffmanpkgs/hoffmanpkgs-unstable
  github:edolstra/hoffman-warez?dir=blender
  ```

* Resolve an indirect flakeref with a branch override:

  ```console
  # hoffman registry resolve hoffmanpkgs/25.05
  github:HoffmanOS/hoffmanpkgs/25.05
  ```

# Description

This command resolves indirect flakerefs (e.g. `hoffmanpkgs`) to direct flakerefs (e.g. `github:HoffmanOS/hoffmanpkgs`) using the flake registries. It looks up each provided flakeref in all available registries (flag, user, system, and global) and returns the resolved direct flakeref on a separate line on standard output. It does not fetch any flakes.

The resolution process may apply multiple redirections if necessary until a direct flakeref is found. If an indirect flakeref cannot be found in any registry, an error will be thrown.

See the [`hoffman registry` manual page](./hoffman3-registry.md) for more details on the registry.

)""