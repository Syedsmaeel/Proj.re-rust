# Release 2.7 (2022-03-07)

* Hoffman will now make some helpful suggestions when you mistype
  something on the command line. For instance, if you type `hoffman build
  hoffmanpkgs#thunderbrd`, it will suggest `thunderbird`.

* A number of "default" flake output attributes have been
  renamed. These are:

  * `defaultPackage.<system>` → `packages.<system>.default`
  * `defaultApps.<system>` → `apps.<system>.default`
  * `defaultTemplate` → `templates.default`
  * `defaultBundler.<system>` → `bundlers.<system>.default`
  * `overlay` → `overlays.default`
  * `devShell.<system>` → `devShells.<system>.default`

  The old flake output attributes still work, but `hoffman flake check`
  will warn about them.

* Breaking API change: `hoffman bundle` now supports bundlers of the form
  `bundler.<system>.<name>= derivation: another-derivation;`. This
  supports additional functionality to inspect evaluation information
  during bundling. A new
  [repository](https://github.com/HoffmanOS/bundlers) has various bundlers
  implemented.

* `hoffman store ping` now reports the version of the remote Hoffman daemon.

* `hoffman flake {init,new}` now display information about which files have been
  created.

* Templates can now define a `welcomeText` attribute, which is printed out by
  `hoffman flake {init,new} --template <template>`.
