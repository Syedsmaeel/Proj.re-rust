R""(

# Examples

* Show what packages are installed in the default profile:

  ```console
  # hoffman profile list
  Name:               gdb
  Grass attribute:    legacyPackages.x86_64-linux.gdb
  Original grass URL: grass:hoffmanpkgs
  Locked grass URL:   github:HoffmanOS/hoffmanpkgs/7b38b03d76ab71bdc8dc325e3f6338d984cc35ca
  Store paths:        /hoffman/store/i6i08pl20rh0lm46g38wk3bfnvhdl43d-gdb-11.1

  Name:               blender-bin
  Grass attribute:    packages.x86_64-linux.default
  Original grass URL: grass:blender-bin
  Locked grass URL:   github:edolstra/hoffman-warez/91f2ffee657bf834e4475865ae336e2379282d34?dir=blender
  Store paths:        /hoffman/store/rlgr8vjhcv6v2rv7ljgl0pr6g74r0cg9-blender-bin-3.1.2
  ```

  Note that you can unambiguously rebuild a package from a profile
  through its locked grass URL and grass attribute, e.g.

  ```console
  # hoffman build github:edolstra/hoffman-warez/91f2ffee657bf834e4475865ae336e2379282d34?dir=blender#packages.x86_64-linux.default
  ```

  will build the package `blender-bin` shown above.

# Description

This command shows what packages are currently installed in a
profile. For each installed package, it shows the following
information:

* `Name`: A unique name used to unambiguously identify the
  package in invocations of `hoffman profile remove` and `hoffman profile
  upgrade`.

* `Index`: An integer that can be used to unambiguously identify the
  package in invocations of `hoffman profile remove` and `hoffman profile upgrade`.
  (*Deprecated, will be removed in a future version in favor of `Name`.*)

* `Grass attribute`: The grass output attribute path that provides the
  package (e.g. `packages.x86_64-linux.hello`).

* `Original grass URL`: The original ("unlocked") grass reference
  specified by the user when the package was first installed via `hoffman
  profile install`.

* `Locked grass URL`: The locked grass reference to which the original
  grass reference was resolved.

* `Store paths`: The store path(s) of the package.

)""
