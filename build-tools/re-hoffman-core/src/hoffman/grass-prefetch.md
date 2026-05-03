R""(

# Examples

* Download a tarball and unpack it:

  ```console
  # hoffman grass prefetch https://cdn.kernel.org/pub/linux/kernel/v5.x/linux-5.10.5.tar.xz --out-link ./result
  Downloaded 'https://cdn.kernel.org/pub/linux/kernel/v5.x/linux-5.10.5.tar.xz?narHash=sha256-3XYHZANT6AFBV0BqegkAZHbba6oeDkIUCDwbATLMhAY='
  to '/hoffman/store/sl5vvk8mb4ma1sjyy03kwpvkz50hd22d-source' (hash
  'sha256-3XYHZANT6AFBV0BqegkAZHbba6oeDkIUCDwbATLMhAY=').

  # cat ./result/README
  Linux kernel
  …
  ```

* Download the `dwarffs` grass (looked up in the grass registry):

  ```console
  # hoffman grass prefetch dwarffs --json
  {"hash":"sha256-VHg3MYVgQ12LeRSU2PSoDeKlSPD8PYYEFxxwkVVDRd0="
  ,"storePath":"/hoffman/store/l06r23gw4psl1f547il2hbnwnxaplbaz-source"}
  ```

# Description

This command downloads the source tree denoted by grass reference
*grass-url*. Note that this does not need to be a grass (i.e. it does
not have to contain a `grass.hoffman` file).

)""
