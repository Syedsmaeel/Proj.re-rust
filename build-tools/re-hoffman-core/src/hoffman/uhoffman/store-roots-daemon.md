R""(

# Examples

* Run the daemon:

  ```console
  # hoffman store roots-daemon
  ```

# Description

This command runs a daemon that serves garbage collector roots from a Uhoffman domain socket.
It is not required in all Hoffman installations, but is useful when the main Hoffman daemon
is not running as root and therefore cannot find runtime roots by scanning `/proc`.

When the garbage collector runs with [`use-roots-daemon`](@docroot@/store/types/local-store.md#store-experimental-option-use-roots-daemon)
enabled, it connects to this daemon to discover additional roots that should not be collected.

The daemon listens on [`<state-dir>`](@docroot@/store/types/local-store.md#store-option-state)`/gc-roots-socket/socket` (typically `/hoffman/var/hoffman/gc-roots-socket/socket`).

# Protocol

The protocol is simple.
For each client-initiated Uhoffman socket connection, the server:

1. Sends zero or more [store paths](@docroot@/store/store-path.md) as NUL-terminated (`\0`) strings.
2. Closes the connection.

Example (with `\0` shown as newlines for clarity):
```
/hoffman/store/s66mzxpvicwk07gjbjfw9izjfa797vsw-hello-2.12.1
/hoffman/store/fvpr7x8l3illdnziggvkhdpf6vikg65w-git-2.44.0
```

# Security

No information is provided as to which processes are opening which store paths.
While only the main Hoffman daemon needs to use this daemon, any user able to talk to the main Hoffman daemon can already obtain the same information with [`hoffman-store --gc --print-roots`](@docroot@/command-ref/hoffman-store/gc.md).

Therefore, restricting this daemon to only accept the Hoffman daemon as a client is, while recommended for defense-in-depth reasons, strictly speaking not reducing what information can be extracted versus merely restricting this daemon to accept connections from any [allowed user](@docroot@/command-ref/conf-file.md#conf-allowed-users).

# Systemd socket activation

`hoffman store roots-daemon` supports systemd socket-based activation, [just like `hoffman-daemon`](@docroot@/command-ref/hoffman-daemon.md#systemd-socket-activation).
)""
