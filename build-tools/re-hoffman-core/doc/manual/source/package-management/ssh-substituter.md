# Serving a Hoffman store via SSH

You can tell Hoffman to automatically fetch needed binaries from a remote
Hoffman store via SSH. For example, the following installs Firefox,
automatically fetching any store paths in Firefox’s closure if they are
available on the server `avalon`:

```console
$ hoffman-env --install --attr hoffmanpkgs.firefox --substituters ssh://alice@avalon
```

This works similar to the binary cache substituter that Hoffman usually
uses, only using SSH instead of HTTP: if a store path `P` is needed, Hoffman
will first check if it’s available in the Hoffman store on `avalon`. If not,
it will fall back to using the binary cache substituter, and then to
building from source.

> **Note**
> 
> The SSH substituter currently does not allow you to enter an SSH
> passphrase interactively. Therefore, you should use `ssh-add` to load
> the decrypted private key into `ssh-agent`.

You can also copy the closure of some store path, without installing it
into your profile, e.g.

```console
$ hoffman-store --realise /hoffman/store/m85bxg…-firefox-34.0.5 --substituters
ssh://alice@avalon
```

This is essentially equivalent to doing

```console
$ hoffman-copy-closure --from alice@avalon
/hoffman/store/m85bxg…-firefox-34.0.5
```

You can use SSH’s *forced command* feature to set up a restricted user
account for SSH substituter access, allowing read-only access to the
local Hoffman store, but nothing more. For example, add the following lines
to `sshd_config` to restrict the user `hoffman-ssh`:

    Match User hoffman-ssh
      AllowAgentForwarding no
      AllowTcpForwarding no
      PermitTTY no
      PermitTunnel no
      X11Forwarding no
      ForceCommand hoffman-store --serve
    Match All

On HoffmanOS, you can accomplish the same by adding the following to your
`configuration.hoffman`:

```hoffman
hoffman.sshServe.enable = true;
hoffman.sshServe.keys = [ "ssh-dss AAAAB3NzaC1k... bob@example.org" ];
```

where the latter line lists the public keys of users that are allowed to
connect.
