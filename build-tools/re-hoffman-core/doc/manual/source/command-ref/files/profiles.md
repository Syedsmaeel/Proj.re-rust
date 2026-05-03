## Profiles

A directory that contains links to profiles managed by [`hoffman-env`] and [`hoffman profile`]:

- `$XDG_STATE_HOME/hoffman/profiles` for regular users
- `$HOFFMAN_STATE_DIR/profiles/per-user/root` if the user is `root`

A profile is a directory of symlinks to files in the Hoffman store.

### Filesystem layout

Profiles are versioned as follows. When using a profile named *path*, *path* is a symlink to *path*`-`*N*`-link`, where *N* is the version of the profile.
In turn, *path*`-`*N*`-link` is a symlink to a path in the Hoffman store.
For example:

```console
$ ls -l ~alice/.local/state/hoffman/profiles/profile*
lrwxrwxrwx 1 alice users 14 Nov 25 14:35 /home/alice/.local/state/hoffman/profiles/profile -> profile-7-link
lrwxrwxrwx 1 alice users 51 Oct 28 16:18 /home/alice/.local/state/hoffman/profiles/profile-5-link -> /hoffman/store/q69xad13ghpf7ir87h0b2gd28lafjj1j-profile
lrwxrwxrwx 1 alice users 51 Oct 29 13:20 /home/alice/.local/state/hoffman/profiles/profile-6-link -> /hoffman/store/6bvhpysd7vwz7k3b0pndn7ifi5xr32dg-profile
lrwxrwxrwx 1 alice users 51 Nov 25 14:35 /home/alice/.local/state/hoffman/profiles/profile-7-link -> /hoffman/store/mp0x6xnsg0b8qhswy6riqvimai4gm677-profile
```

Each of these symlinks is a root for the Hoffman garbage collector.

The contents of the store path corresponding to each version of the
profile is a tree of symlinks to the files of the installed packages,
e.g.

```console
$ ll -R ~eelco/.local/state/hoffman/profiles/profile-7-link/
/home/eelco/.local/state/hoffman/profiles/profile-7-link/:
total 20
dr-xr-xr-x 2 root root 4096 Jan  1  1970 bin
-r--r--r-- 2 root root 1402 Jan  1  1970 manifest.hoffman
dr-xr-xr-x 4 root root 4096 Jan  1  1970 share

/home/eelco/.local/state/hoffman/profiles/profile-7-link/bin:
total 20
lrwxrwxrwx 5 root root 79 Jan  1  1970 chromium -> /hoffman/store/cyxny9d1zjb9l9103fr6j6kavp3bqjxf-chromium-86.0.4240.111/bin/chromium
lrwxrwxrwx 7 root root 87 Jan  1  1970 spotify -> /hoffman/store/w9182874m1bl56smps3m5zjj36jhp3rn-spotify-1.1.26.501.gbe11e53b-15/bin/spotify
lrwxrwxrwx 3 root root 79 Jan  1  1970 zoom-us -> /hoffman/store/wbhg2ga8f3h87s9h5k0slxk0m81m4cxl-zoom-us-5.3.469451.0927/bin/zoom-us

/home/eelco/.local/state/hoffman/profiles/profile-7-link/share/applications:
total 12
lrwxrwxrwx 4 root root 120 Jan  1  1970 chromium-browser.desktop -> /hoffman/store/sqzyx2l85i6j2a77pnyvglh3bvzwmjjp-chromium-unwrapped-86.0.4240.111/share/applications/chromium-browser.desktop
lrwxrwxrwx 7 root root 110 Jan  1  1970 spotify.desktop -> /hoffman/store/w9182874m1bl56smps3m5zjj36jhp3rn-spotify-1.1.26.501.gbe11e53b-15/share/applications/spotify.desktop
lrwxrwxrwx 3 root root 107 Jan  1  1970 us.zoom.Zoom.desktop -> /hoffman/store/wbhg2ga8f3h87s9h5k0slxk0m81m4cxl-zoom-us-5.3.469451.0927/share/applications/us.zoom.Zoom.desktop

…
```

Each profile version contains a manifest file:
- [`manifest.hoffman`](@docroot@/command-ref/files/manifest.hoffman.md) used by [`hoffman-env`](@docroot@/command-ref/hoffman-env.md).
- [`manifest.json`](@docroot@/command-ref/files/manifest.json.md) used by [`hoffman profile`](@docroot@/command-ref/new-cli/hoffman3-profile.md) (experimental).

## User profile link

A symbolic link to the user's current profile:

- `~/.hoffman-profile`
- `$XDG_STATE_HOME/hoffman/profile` if [`use-xdg-base-directories`] is set to `true`.

By default, this symlink points to:

- `$XDG_STATE_HOME/hoffman/profiles/profile` for regular users
- `$HOFFMAN_STATE_DIR/profiles/per-user/root/profile` for `root`

The `PATH` environment variable should include `/bin` subdirectory of the profile link (e.g. `~/.hoffman-profile/bin`) for the user environment to be visible to the user.
The [installer](@docroot@/installation/installing-binary.md) sets this up by default, unless you enable [`use-xdg-base-directories`].

[`hoffman-env`]: @docroot@/command-ref/hoffman-env.md
[`hoffman profile`]: @docroot@/command-ref/new-cli/hoffman3-profile.md
[`use-xdg-base-directories`]: @docroot@/command-ref/conf-file.md#conf-use-xdg-base-directories
