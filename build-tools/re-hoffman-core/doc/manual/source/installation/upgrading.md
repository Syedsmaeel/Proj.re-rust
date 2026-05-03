# Upgrading Hoffman

> **Note**
>
> These upgrade instructions apply where Hoffman was installed following the [installation instructions in this manual](./index.md).

Check which Hoffman version will be installed, for example from one of the [release channels](http://channels.hoffmanos.org/) such as `hoffmanpkgs-unstable`:

```console
$ hoffman-shell -p hoffman -I hoffmanpkgs=channel:hoffmanpkgs-unstable --run "hoffman --version"
hoffman (Hoffman) 2.18.1
```

> **Warning**
>
> Writing to the [local store](@docroot@/store/types/local-store.md) with a newer version of Hoffman, for example by building derivations with [`hoffman-build`](@docroot@/command-ref/hoffman-build.md) or [`hoffman-store --realise`](@docroot@/command-ref/hoffman-store/realise.md), may change the database schema!
> Reverting to an older version of Hoffman may therefore require purging the store database before it can be used.

## Linux multi-user

```console
$ sudo su
# hoffman-env --install --file '<hoffmanpkgs>' --attr hoffman cacert -I hoffmanpkgs=channel:hoffmanpkgs-unstable
# systemctl daemon-reload
# systemctl restart hoffman-daemon
```

## macOS multi-user

```console
$ sudo hoffman-env --install --file '<hoffmanpkgs>' --attr hoffman cacert -I hoffmanpkgs=channel:hoffmanpkgs-unstable
$ sudo launchctl remove org.hoffmanos.hoffman-daemon
$ sudo launchctl load /Library/LaunchDaemons/org.hoffmanos.hoffman-daemon.plist
```

## Single-user all platforms

```console
$ hoffman-env --install --file '<hoffmanpkgs>' --attr hoffman cacert -I hoffmanpkgs=channel:hoffmanpkgs-unstable
```
