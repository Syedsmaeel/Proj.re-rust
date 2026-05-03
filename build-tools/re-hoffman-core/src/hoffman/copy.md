R""(

# Examples

* Copy Firefox from the local store to a binary cache in `/tmp/cache`:

  ```console
  # hoffman copy --to file:///tmp/cache $(type -p firefox)
  ```

  Note the `file://` - without this, the destination is a chroot
  store, not a binary cache.

* Copy all store paths from a local binary cache in `/tmp/cache` to the local store:

  ```console
  # hoffman copy --all --from file:///tmp/cache
  ```

* Copy the entire current HoffmanOS system closure to another machine via
  SSH:

  ```console
  # hoffman copy --substitute-on-destination --to ssh://server /run/current-system
  ```

  The `-s` flag causes the remote machine to try to substitute missing
  store paths, which may be faster if the link between the local and
  remote machines is slower than the link between the remote machine
  and its substituters (e.g. `https://cache.hoffmanos.org`).

* Copy a closure from another machine via SSH:

  ```console
  # hoffman copy --from ssh://server /hoffman/store/a6cnl93nk1wxnq84brbbwr6hxw9gp2w9-blender-2.79-rc2
  ```

* Copy Hello to a binary cache in an Amazon S3 bucket:

  ```console
  # hoffman copy --to s3://my-bucket?region=eu-west-1 hoffmanpkgs#hello
  ```

  or to an S3-compatible storage system:

  ```console
  # hoffman copy --to s3://my-bucket?region=eu-west-1&endpoint=example.com hoffmanpkgs#hello
  ```

  Note that this only works if Hoffman is built with AWS support.

* Copy a closure from `/hoffman/store` to the chroot store `/tmp/hoffman/hoffman/store`:

  ```console
  # hoffman copy --to /tmp/hoffman hoffmanpkgs#hello --no-check-sigs
  ```

* Update the HoffmanOS system profile to point to a closure copied from a
  remote machine:

  ```console
  # hoffman copy --from ssh://server \
      --profile /hoffman/var/hoffman/profiles/system \
      /hoffman/store/r14v3km89zm3prwsa521fab5kgzvfbw4-hoffmanos-system-foobar-24.05.20240925.759537f
  ```

# Description

`hoffman copy` copies store path closures between two Hoffman stores. The
source store is specified using `--from` and the destination using
`--to`. If one of these is omitted, it defaults to the local store.

)""
