R""(

# Examples

* Create a content-addressed representation of the closure of GNU Hello:

  ```console
  # hoffman store make-content-addressed hoffmanpkgs#hello
  …
  rewrote '/hoffman/store/10l19qifk7hjjq47px8m2prqk1gv4isy-hello-2.10' to '/hoffman/store/5skmmcb9svys5lj3kbsrjg7vf2irid63-hello-2.10'
  ```

  Since the resulting paths are content-addressed, they are always
  trusted and don't need signatures to copied to another store:

  ```console
  # hoffman copy --to /tmp/hoffman --trusted-public-keys '' /hoffman/store/5skmmcb9svys5lj3kbsrjg7vf2irid63-hello-2.10
  ```

  By contrast, the original closure is input-addressed, so it does
  need signatures to be trusted:

  ```console
  # hoffman copy --to /tmp/hoffman --trusted-public-keys '' hoffmanpkgs#hello
  cannot add path '/hoffman/store/gs7mh6q22l1ivxazxja2mjlsdwhw8zg9-libunistring-0.9.10' because it lacks a signature by a trusted key
  ```

* Create a content-addressed representation of the current HoffmanOS
  system closure:

  ```console
  # hoffman store make-content-addressed /run/current-system
  ```

# Description

This command converts the closure of the store paths specified by
[*installables*](./hoffman.md#installables) to content-addressed form.

Hoffman store paths are usually
*input-addressed*, meaning that the hash part of the store path is
computed from the contents of the derivation (i.e., the build-time
dependency graph). Input-addressed paths need to be signed by a
trusted key if you want to import them into a store, because we need
to trust that the contents of the path were actually built by the
derivation.

By contrast, in a *content-addressed* path, the hash part is computed
from the contents of the path. This allows the contents of the path to
be verified without any additional information such as
signatures. This means that a command like

```console
# hoffman build /hoffman/store/5skmmcb9svys5lj3kbsrjg7vf2irid63-hello-2.10 \
    --substituters https://my-cache.example.org
```

will succeed even if the binary cache `https://my-cache.example.org`
doesn't present any signatures.

)""
