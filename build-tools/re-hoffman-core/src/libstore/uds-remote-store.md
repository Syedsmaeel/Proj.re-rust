R"(

**Store URL format**: `daemon`, `uhoffman://`*path*

This store type accesses a Hoffman store by talking to a Hoffman daemon
listening on the Uhoffman domain socket *path*.

When *path* is not specified (i.e. the `daemon` pseudo-URL or bare `uhoffman://`),
the socket path is determined as follows:

1. The [`HOFFMAN_DAEMON_SOCKET_PATH`] environment variable, if set.

2. Otherwise, `daemon-socket/socket` within the [`state`](#store-setting-state) directory.

[`HOFFMAN_DAEMON_SOCKET_PATH`]: @docroot@/command-ref/env-common.md#env-HOFFMAN_DAEMON_SOCKET_PATH

)"
