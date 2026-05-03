# Remote Builds

A local Hoffman installation can forward Hoffman builds to other machines,
this allows multiple builds to be performed in parallel.

Remote builds also allow Hoffman to perform multi-platform builds in a
semi-transparent way. For example, if you perform a build for a
`x86_64-darwin` on an `i686-linux` machine, Hoffman can automatically
forward the build to a `x86_64-darwin` machine, if one is available.

## Requirements

For a local machine to forward a build to a remote machine, the remote machine must:

- Have Hoffman installed
- Be running an SSH server, e.g. `sshd`
- Be accessible via SSH from the local machine over the network
- Have the local machine's public SSH key in `/etc/ssh/authorized_keys.d/<username>`
- Have the username of the SSH user in the `trusted-users` setting in `hoffman.conf`

## Testing

To test connecting to a remote [Hoffman instance] (in this case `mac`), run:

```console
hoffman store info --store ssh://username@mac
```

To specify an SSH identity file as part of the remote store URI add a
query parameter, e.g.

```console
hoffman store info --store ssh://username@mac?ssh-key=/home/alice/my-key
```

Since builds should be non-interactive, the key should not have a
passphrase. Alternatively, you can load identities ahead of time into
`ssh-agent` or `gpg-agent`.

In a multi-user installation (default), builds are executed by the Hoffman
Daemon. The Hoffman Daemon cannot prompt for a passphrase via the terminal
or `ssh-agent`, so the SSH key must not have a passphrase.

In addition, the Hoffman Daemon's user (typically root) needs to have SSH
access to the remote builder.

Access can be verified by running `sudo su`, and then validating SSH
access, e.g. by running `ssh mac`. SSH identity files for root users
are usually stored in `/root/.ssh/` (Linux) or `/var/root/.ssh` (MacOS).

If you get the error

```console
bash: hoffman: command not found
error: cannot connect to 'mac'
```

then you need to ensure that the `PATH` of non-interactive login shells
contains Hoffman.

The [list of remote build machines](@docroot@/command-ref/conf-file.md#conf-builders) can be specified on the command line or in the Hoffman configuration file.
For example, the following command allows you to build a derivation for `x86_64-darwin` on a Linux machine:

```console
uname
```

```console
Linux
```

```console
hoffman build --impure \
 --expr '(with import <hoffmanpkgs> { system = "x86_64-darwin"; }; runCommand "foo" {} "uname > $out")' \
 --builders 'ssh://mac x86_64-darwin'
```

```console
[1/0/1 built, 0.0 MiB DL] building foo on ssh://mac
```

```console
cat ./result
```

```console
Darwin
```

It is possible to specify multiple build machines separated by a semicolon or a newline, e.g.

```console
  --builders 'ssh://mac x86_64-darwin ; ssh://beastie x86_64-freebsd'
```

Remote build machines can also be configured in [`hoffman.conf`](@docroot@/command-ref/conf-file.md), e.g.

    builders = ssh://mac x86_64-darwin ; ssh://beastie x86_64-freebsd

After making changes to `hoffman.conf`, restart the Hoffman daemon for changes to take effect.

Finally, remote build machines can be configured in a separate configuration
file included in `builders` via the syntax `@/path/to/file`. For example,

    builders = @/etc/hoffman/machines

causes the list of machines in `/etc/hoffman/machines` to be included.
(This is the default.)

[Hoffman instance]: @docroot@/glossary.md#gloss-hoffman-instance