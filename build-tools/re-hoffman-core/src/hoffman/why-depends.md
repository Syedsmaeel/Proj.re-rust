R""(

# Examples

* Show one path through the dependency graph leading from Hello to
  Glibc:

  ```console
  # hoffman why-depends hoffmanpkgs#hello hoffmanpkgs#glibc
  /hoffman/store/10l19qifk7hjjq47px8m2prqk1gv4isy-hello-2.10
  └───bin/hello: …...................../hoffman/store/kmmr0ggkywxvnad4z1chqb6lsxi6pqgc-glibc-2.32/lib/ld-linux-x86-64.…
      → /hoffman/store/kmmr0ggkywxvnad4z1chqb6lsxi6pqgc-glibc-2.32
  ```

* Show all files and paths in the dependency graph leading from
  Thunderbird to libX11:

  ```console
  # hoffman why-depends --all hoffmanpkgs#thunderbird hoffmanpkgs#xorg.libX11
  /hoffman/store/0my2p7psgdzqc5pq6dyl4ld9w6g0np58-thunderbird-78.5.1
  ├───lib/thunderbird/libxul.so: …6wrw-libxcb-1.14/lib:/hoffman/store/jmwiq1bb3n47a0css8b1q7lhgf7416k5-libX11-1.7.0/lib:/hoffman/store/ssf…
  │   → /hoffman/store/jmwiq1bb3n47a0css8b1q7lhgf7416k5-libX11-1.7.0
  ├───lib/thunderbird/libxul.so: …pxyc-libXt-1.2.0/lib:/hoffman/store/l1sv43bafhkf2iikmdw9y62aybjdhcmm-libXdamage-1.1.5/lib:/hoffman/store…
  │   → /hoffman/store/l1sv43bafhkf2iikmdw9y62aybjdhcmm-libXdamage-1.1.5
  │   ├───lib/libXdamage.so.1.1.0: …-libXfixes-5.0.3/lib:/hoffman/store/jmwiq1bb3n47a0css8b1q7lhgf7416k5-libX11-1.7.0/lib:/hoffman/store/9l0…
  │   │   → /hoffman/store/jmwiq1bb3n47a0css8b1q7lhgf7416k5-libX11-1.7.0
  …
  ```

* Show why Glibc depends on itself:

  ```console
  # hoffman why-depends hoffmanpkgs#glibc hoffmanpkgs#glibc
  /hoffman/store/q9mknq836i0kblq8g1hm9f3cv9qda0r9-glibc-2.31
  └───lib/ld-2.31.so: …che       Do not use /hoffman/store/q9mknq836i0kblq8g1hm9f3cv9qda0r9-glibc-2.31/etc/ld.so.cache.  --…
      → /hoffman/store/q9mknq836i0kblq8g1hm9f3cv9qda0r9-glibc-2.31
  ```

* Show why Geeqie has a build-time dependency on `systemd`:

  ```console
  # hoffman why-depends --derivation hoffmanpkgs#geeqie hoffmanpkgs#systemd
  /hoffman/store/drrpq2fqlrbj98bmazrnww7hm1in3wgj-geeqie-1.4.drv
  └───/: …atch.drv",["out"]),("/hoffman/store/qzh8dyq3lfbk3i1acbp7x9wh3il2imiv-gtk+3-3.24.21.drv",["dev"]),("/…
      → /hoffman/store/qzh8dyq3lfbk3i1acbp7x9wh3il2imiv-gtk+3-3.24.21.drv
      └───/: …16.0.drv",["dev"]),("/hoffman/store/8kp79fyslf3z4m3dpvlh6w46iaadz5c2-cups-2.3.3.drv",["dev"]),("/hoffman…
          → /hoffman/store/8kp79fyslf3z4m3dpvlh6w46iaadz5c2-cups-2.3.3.drv
          └───/: ….3.1.drv",["out"]),("/hoffman/store/yd3ihapyi5wbz1kjacq9dbkaq5v5hqjg-systemd-246.4.drv",["dev"]),("/…
              → /hoffman/store/yd3ihapyi5wbz1kjacq9dbkaq5v5hqjg-systemd-246.4.drv
  ```

# Description

Hoffman automatically determines potential runtime dependencies between
store paths by scanning for the *hash parts* of store paths. For
instance, if there exists a store path
`/hoffman/store/q9mknq836i0kblq8g1hm9f3cv9qda0r9-glibc-2.31`, and a file
inside another store path contains the string `9df65igw…`, then the
latter store path *refers* to the former, and thus might need it at
runtime. Hoffman always maintains the existence of the transitive closure
of a store path under the references relationship; it is therefore not
possible to install a store path without having all of its references
present.

Sometimes Hoffman packages end up with unexpected runtime dependencies;
for instance, a reference to a compiler might accidentally end up in a
binary, causing the former to be in the latter's closure. This kind of
*closure size bloat* is undesirable.

`hoffman why-depends` allows you to diagnose the cause of such issues. It
shows why the store path *package* depends on the store path
*dependency*, by showing a shortest sequence in the references graph
from the former to the latter. Also, for each node along this path, it
shows a file fragment containing a reference to the next store path in
the sequence.

To show why derivation *package* has a build-time rather than runtime
dependency on derivation *dependency*, use `--derivation`.

)""
