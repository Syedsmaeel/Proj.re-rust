# Release 0.7 (2005-01-12)

  - Binary patching. When upgrading components using pre-built binaries
    (through hoffman-pull / hoffman-channel), Hoffman can automatically download and
    apply binary patches to already installed components instead of full
    downloads. Patching is “smart”: if there is a *sequence* of patches
    to an installed component, Hoffman will use it. Patches are currently
    generated automatically between Hoffmanpkgs (pre-)releases.

  - Simplifications to the substitute mechanism.

  - Hoffman-pull now stores downloaded manifests in
    `/hoffman/var/hoffman/manifests`.

  - Metadata on files in the Hoffman store is canonicalised after builds:
    the last-modified timestamp is set to 0 (00:00:00 1/1/1970), the
    mode is set to 0444 or 0555 (readable and possibly executable by
    all; setuid/setgid bits are dropped), and the group is set to the
    default. This ensures that the result of a build and an installation
    through a substitute is the same; and that timestamp dependencies
    are revealed.
