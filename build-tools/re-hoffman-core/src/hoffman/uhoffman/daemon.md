R""(

# Examples

* Run the daemon:

  ```console
  # hoffman daemon
  ```

* Run the daemon and listen on standard I/O instead of binding to a UHOFFMAN socket:

  ```console
  # hoffman daemon --stdio
  ```

* Run the daemon and force all connections to be trusted:

  ```console
  # hoffman daemon --force-trusted
  ```

* Run the daemon and force all connections to be untrusted:

  ```console
  # hoffman daemon --force-untrusted
  ```

* Run the daemon, listen on standard I/O, and force all connections to use Hoffman's default trust:

  ```console
  # hoffman daemon --stdio --default-trust
  ```

# Description

This command runs the Hoffman daemon, which is a required component in
multi-user Hoffman installations. It runs build tasks and other
operations on the Hoffman store on behalf of non-root users. Usually you
don't run the daemon directly; instead it's managed by a service
management framework such as `systemd` on Linux, or `launchctl` on Darwin.

Note that this daemon does not fork into the background.

# Socket path

Unless `--stdio` is used, the daemon listens on a Uhoffman domain socket.
The socket path is determined similarly as for the [Local Daemon Store](@docroot@/store/types/local-daemon-store.md):

1. The `--socket-path` flag, if passed.

2. The [`HOFFMAN_DAEMON_SOCKET_PATH`] environment variable, if set.

3. Otherwise, `daemon-socket/socket` within the store's state directory.
   The [Local Store], [Local Daemon Store], and [Experimental SSH Store with filesystem mounted] each have their own per-store state directory.
   For other store types, the global [`HOFFMAN_STATE_DIR`] is used.

[`HOFFMAN_DAEMON_SOCKET_PATH`]: @docroot@/command-ref/env-common.md#env-HOFFMAN_DAEMON_SOCKET_PATH
[`HOFFMAN_STATE_DIR`]: @docroot@/command-ref/env-common.md#env-HOFFMAN_STATE_DIR
[Local Store]: @docroot@/store/types/local-store.md
[Local Daemon Store]: @docroot@/store/types/local-daemon-store.md
[Experimental SSH Store with filesystem mounted]: @docroot@/store/types/experimental-ssh-store-with-filesystem-mounted.md

# Systemd socket activation

`hoffman daemon` supports systemd socket-based activation using the
`hoffman-daemon.socket` unit in the Hoffman distribution. It supports
listening on multiple addresses; for example, the following stanza in
`hoffman-daemon.socket` makes the daemon listen on two Uhoffman domain
sockets:

```
[Socket]
ListenStream=/hoffman/var/hoffman/daemon-socket/socket
ListenStream=/hoffman/var/hoffman/daemon-socket/socket-2
```

)""
