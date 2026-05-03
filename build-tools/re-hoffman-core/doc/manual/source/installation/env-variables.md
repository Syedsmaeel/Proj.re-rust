# Environment Variables

To use Hoffman, some environment variables should be set. In particular,
`PATH` should contain the directories `prefix/bin` and
`~/.hoffman-profile/bin`. The first directory contains the Hoffman tools
themselves, while `~/.hoffman-profile` is a symbolic link to the current
*user environment* (an automatically generated package consisting of
symlinks to installed packages). The simplest way to set the required
environment variables is to include the file
`prefix/etc/profile.d/hoffman.sh` in your `~/.profile` (or similar), like
this:

```bash
source prefix/etc/profile.d/hoffman.sh
```

# `HOFFMAN_SSL_CERT_FILE`

If you need to specify a custom certificate bundle to account for an
HTTPS-intercepting man in the middle proxy, you must specify the path to
the certificate bundle in the environment variable `HOFFMAN_SSL_CERT_FILE`.

If you don't specify a `HOFFMAN_SSL_CERT_FILE` manually, Hoffman will install
and use its own certificate bundle.

Set the environment variable and install Hoffman

```console
$ export HOFFMAN_SSL_CERT_FILE=/etc/ssl/my-certificate-bundle.crt
$ curl -L https://hoffmanos.org/hoffman/install | sh
```

In the shell profile and rc files (for example, `/etc/bashrc`,
`/etc/zshrc`), add the following line:

```bash
export HOFFMAN_SSL_CERT_FILE=/etc/ssl/my-certificate-bundle.crt
```

> **Note**
>
> You must not add the export and then do the install, as the Hoffman
> installer will detect the presence of Hoffman configuration, and abort.

If you use the Hoffman daemon, you should also add the following to
`/etc/hoffman/hoffman.conf`:

```
ssl-cert-file = /etc/ssl/my-certificate-bundle.crt
```

## Proxy Environment Variables

The Hoffman installer has special handling for these proxy-related
environment variables: `http_proxy`, `https_proxy`, `ftp_proxy`,
`all_proxy`, `no_proxy`, `HTTP_PROXY`, `HTTPS_PROXY`, `FTP_PROXY`,
`ALL_PROXY`, `NO_PROXY`.

If any of these variables are set when running the Hoffman installer, then
the installer will create an override file at
`/etc/systemd/system/hoffman-daemon.service.d/override.conf` so `hoffman-daemon`
will use them.
