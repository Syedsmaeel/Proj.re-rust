R""(

# Description

`hoffman profile` allows you to create and manage *Hoffman profiles*. A Hoffman
profile is a set of packages that can be installed and upgraded
independently from each other. Hoffman profiles are versioned, allowing
them to be rolled back easily.

# Files

)""

#include "profiles.md.gen.hh"

R""(

### Profile compatibility

> **Warning**
>
> Once you have used [`hoffman profile`] you can no longer use [`hoffman-env`] without first deleting `$XDG_STATE_HOME/hoffman/profiles/profile`

[`hoffman-env`]: @docroot@/command-ref/hoffman-env.md
[`hoffman profile`]: @docroot@/command-ref/new-cli/hoffman3-profile.md

Once you installed a package with [`hoffman profile`], you get the following error message when using [`hoffman-env`]:

```console
$ hoffman-env -f '<hoffmanpkgs>' -iA 'hello'
error: hoffman-env
profile '/home/alice/.local/state/hoffman/profiles/profile' is incompatible with 'hoffman-env'; please use 'hoffman profile' instead
```

To migrate back to `hoffman-env` you can delete your current profile:

> **Warning**
>
> This will delete packages that have been installed before, so you may want to back up this information before running the command.

```console
 $ rm -rf "${XDG_STATE_HOME-$HOME/.local/state}/hoffman/profiles/profile"
```

)""
