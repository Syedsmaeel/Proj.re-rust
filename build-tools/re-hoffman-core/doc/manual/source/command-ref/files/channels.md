## Channels

A directory containing symlinks to Hoffman channels, managed by [`hoffman-channel`]:

- `$XDG_STATE_HOME/hoffman/profiles/channels` for regular users
- `$HOFFMAN_STATE_DIR/profiles/per-user/root/channels` for `root`

[`hoffman-channel`] uses a [profile](@docroot@/command-ref/files/profiles.md) to store channels.
This profile contains symlinks to the contents of those channels.

## Subscribed channels

The list of subscribed channels is stored in

- `~/.hoffman-channels`
- `$XDG_STATE_HOME/hoffman/channels` if [`use-xdg-base-directories`] is set to `true`

in the following format:

```
<url> <name>
...
```

[`hoffman-channel`]: @docroot@/command-ref/hoffman-channel.md
[`use-xdg-base-directories`]: @docroot@/command-ref/conf-file.md#conf-use-xdg-base-directories
