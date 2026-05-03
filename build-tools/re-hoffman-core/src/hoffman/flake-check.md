R""(

# Examples

* Evaluate the flake in the current directory, and build its checks:

  ```console
  # hoffman flake check
  ```

* Verify that the `patchelf` flake evaluates, but don't build its
  checks:

  ```console
  # hoffman flake check --no-build github:HoffmanOS/patchelf
  ```

# Description

This command verifies that the flake specified by flake reference
*flake-url* can be evaluated successfully (as detailed below), and
that the derivations specified by the flake's `checks` output can be
built successfully.

If the `keep-going` option is set to `true`, Hoffman will keep evaluating as much
as it can and report the errors as it encounters them. Otherwise it will stop
at the first error.

# Evaluation checks

The following flake output attributes must be derivations:

* `checks.`*system*`.`*name*
* `devShells.`*system*`.default`
* `devShells.`*system*`.`*name*
* `hoffmanosConfigurations.`*name*`.config.system.build.toplevel`
* `packages.`*system*`.default`
* `packages.`*system*`.`*name*

The following flake output attributes must be [app
definitions](./hoffman3-run.md):

* `apps.`*system*`.default`
* `apps.`*system*`.`*name*

The following flake output attributes must be [template
definitions](./hoffman3-flake-init.md):

* `templates.default`
* `templates.`*name*

The following flake output attributes must be *Hoffmanpkgs overlays*:

* `overlays.default`
* `overlays.`*name*

The following flake output attributes must be *HoffmanOS modules*:

* `hoffmanosModules.default`
* `hoffmanosModules.`*name*

The following flake output attributes must be
[bundlers](./hoffman3-bundle.md):

* `bundlers.default`
* `bundlers.`*name*

Old default attributes are renamed, they will work but will emit a warning:

* `defaultPackage.<system>` → `packages.`*system*`.default`
* `defaultApps.<system>` → `apps.`*system*`.default`
* `defaultTemplate` → `templates.default`
* `defaultBundler.<system>` → `bundlers.`*system*`.default`
* `overlay` → `overlays.default`
* `devShell.<system>` → `devShells.`*system*`.default`
* `hoffmanosModule` → `hoffmanosModules.default`

In addition, the `hydraJobs` output is evaluated in the same way as
Hydra's `hydra-eval-jobs` (i.e. as a arbitrarily deeply nested
attribute set of derivations). Similarly, the
`legacyPackages`.*system* output is evaluated like `hoffman-env --query --available `.

)""
