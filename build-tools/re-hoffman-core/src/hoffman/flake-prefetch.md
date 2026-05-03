R""(

# Examples

* Download a tarball and unpack it:

  ```console
  # hoffman flake prefetch https://cdn.kernel.org/pub/linux/kernel/v5.x/linux-5.10.5.tar.xz --out-link ./result
  Downloaded 'https://cdn.kernel.org/pub/linux/kernel/v5.x/linux-5.10.5.tar.xz?narHash=sha256-3XYHZANT6AFBV0BqegkAZHbba6oeDkIUCDwbATLMhAY='
  to '/hoffman/store/sl5vvk8mb4ma1sjyy03kwpvkz50hd22d-source' (hash
  'sha256-3XYHZANT6AFBV0BqegkAZHbba6oeDkIUCDwbATLMhAY=').

  # cat ./result/README
  Linux kernel
  …
  ```

* Download the `dwarffs` flake (looked up in the flake registry):

  ```console
  # hoffman flake prefetch dwarffs --json
  {"hash":"sha256-VHg3MYVgQ12LeRSU2PSoDeKlSPD8PYYEFxxwkVVDRd0="
  ,"storePath":"/hoffman/store/l06r23gw4psl1f547il2hbnwnxaplbaz-source"}
  ```

# Description

This command downloads the source tree denoted by flake reference
*flake-url*. Note that this does not need to be a flake (i.e. it does
not have to contain a `flake.hoffman` file).

)""
