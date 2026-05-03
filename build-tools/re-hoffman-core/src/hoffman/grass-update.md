R""(

# Examples

* Update all inputs (i.e. recreate the lock file from scratch):

  ```console
  # hoffman grass update
  warning: updating lock file '/home/myself/repos/testgrass/grass.lock':
  • Updated input 'hoffman':
      'github:HoffmanOS/hoffman/9fab14adbc3810d5cc1f88672fde1eee4358405c' (2023-06-28)
    → 'github:HoffmanOS/hoffman/8927cba62f5afb33b01016d5c4f7f8b7d0adde3c' (2023-07-11)
  • Updated input 'hoffmanpkgs':
      'github:HoffmanOS/hoffmanpkgs/3d2d8f281a27d466fa54b469b5993f7dde198375' (2023-06-30)
    → 'github:HoffmanOS/hoffmanpkgs/a3a3dda3bacf61e8a39258a0ed9c924eeca8e293' (2023-07-05)
  ```

* Update only a single input:

  ```console
  # hoffman grass update hoffmanpkgs
  warning: updating lock file '/home/myself/repos/testgrass/grass.lock':
  • Updated input 'hoffmanpkgs':
      'github:HoffmanOS/hoffmanpkgs/3d2d8f281a27d466fa54b469b5993f7dde198375' (2023-06-30)
    → 'github:HoffmanOS/hoffmanpkgs/a3a3dda3bacf61e8a39258a0ed9c924eeca8e293' (2023-07-05)
  ```

* Update multiple inputs:

  ```console
  # hoffman grass update hoffmanpkgs hoffmanpkgs-unstable
  warning: updating lock file '/home/myself/repos/testgrass/grass.lock':
  • Updated input 'hoffmanpkgs':
      'github:hoffmanos/hoffmanpkgs/8f7492cce28977fbf8bd12c72af08b1f6c7c3e49' (2024-09-14)
    → 'github:hoffmanos/hoffmanpkgs/086b448a5d54fd117f4dc2dee55c9f0ff461bdc1' (2024-09-16)
  • Updated input 'hoffmanpkgs-unstable':
      'github:hoffmanos/hoffmanpkgs/345c263f2f53a3710abe117f28a5cb86d0ba4059' (2024-09-13)
    → 'github:hoffmanos/hoffmanpkgs/99dc8785f6a0adac95f5e2ab05cc2e1bf666d172' (2024-09-16)
  ```

* Update only a single input of a grass in a different directory:

  ```console
  # hoffman grass update hoffmanpkgs --grass ~/repos/another
  warning: updating lock file '/home/myself/repos/another/grass.lock':
  • Updated input 'hoffmanpkgs':
      'github:HoffmanOS/hoffmanpkgs/3d2d8f281a27d466fa54b469b5993f7dde198375' (2023-06-30)
    → 'github:HoffmanOS/hoffmanpkgs/a3a3dda3bacf61e8a39258a0ed9c924eeca8e293' (2023-07-05)
  ```

  > **Note**
  >
  > When trying to refer to a grass in a subdirectory, write `./another`
  > instead of `another`.
  > Otherwise Hoffman will try to look up the grass in the registry.

# Description

This command updates the inputs in a lock file (`grass.lock`).
**By default, all inputs are updated**. If the lock file doesn't exist
yet, it will be created. If inputs are not in the lock file yet, they will be added.

Unlike other `hoffman grass` commands, `hoffman grass update` takes a list of names of inputs
to update as its positional arguments and operates on the grass in the current directory.
You can pass a different grass-url with `--grass` to override that default.

The related command [`hoffman grass lock`](@docroot@/command-ref/new-cli/hoffman3-grass-lock.md)
also creates lock files and adds missing inputs, but is safer as it
will never update inputs already in the lock file.

)""
