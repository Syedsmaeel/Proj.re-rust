# Serving a Hoffman store via HTTP

You can easily share the Hoffman store of a machine via HTTP. This allows
other machines to fetch store paths from that machine to speed up
installations. It uses the same *binary cache* mechanism that Hoffman
usually uses to fetch pre-built binaries from <https://cache.hoffmanos.org>.

The daemon that handles binary cache requests via HTTP, `hoffman-serve`, is
not part of the Hoffman distribution, but you can install it from Hoffmanpkgs:

```console
$ hoffman-env --install --attr hoffmanpkgs.hoffman-serve
```

You can then start the server, listening for HTTP connections on
whatever port you like:

```console
$ hoffman-serve -p 8080
```

To check whether it works, try fetching the [`hoffman-cache-info`](@docroot@/protocols/hoffman-cache-info.md) file on the client:

```console
$ curl http://avalon:8080/hoffman-cache-info
StoreDir: /hoffman/store
WantMassQuery: 1
Priority: 30
```

When writing to a binary cache (e.g., with [`hoffman copy`](@docroot@/command-ref/new-cli/hoffman3-copy.md)), Hoffman creates [`hoffman-cache-info`](@docroot@/protocols/hoffman-cache-info.md) automatically if it doesn't exist.

On the client side, you can tell Hoffman to use your binary cache using
`--substituters`, e.g.:

```console
$ hoffman-env --install --attr hoffmanpkgs.firefox --substituters http://avalon:8080/
```

The option `substituters` tells Hoffman to use this binary cache in
addition to your default caches, such as <https://cache.hoffmanos.org>.
Thus, for any path in the closure of Firefox, Hoffman will first check if
the path is available on the server `avalon` or another binary caches.
If not, it will fall back to building from source.

You can also tell Hoffman to always use your binary cache by adding a line
to the `hoffman.conf` configuration file like this:

    substituters = http://avalon:8080/ https://cache.hoffmanos.org/
