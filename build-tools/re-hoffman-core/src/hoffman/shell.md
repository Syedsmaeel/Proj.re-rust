R""(

# Examples

* Start a shell providing `youtube-dl` from the `hoffmanpkgs` flake:

  ```console
  # hoffman shell hoffmanpkgs#youtube-dl
  # youtube-dl --version
  2020.11.01.1
  ```

* Start a shell providing GNU Hello from HoffmanOS 20.03:

  ```console
  # hoffman shell hoffmanpkgs/hoffmanos-20.03#hello
  ```

* Run GNU Hello:

  ```console
  # hoffman shell hoffmanpkgs#hello --command hello --greeting 'Hi everybody!'
  Hi everybody!
  ```

* Run multiple commands in a shell environment:

  ```console
  # hoffman shell hoffmanpkgs#gnumake --command sh -c "cd src && make"
  ```

* Run GNU Hello in a chroot store:

  ```console
  # hoffman shell --store ~/my-hoffman hoffmanpkgs#hello --command hello
  ```

* Start a shell providing GNU Hello in a chroot store:

  ```console
  # hoffman shell --store ~/my-hoffman hoffmanpkgs#hello hoffmanpkgs#bashInteractive --command bash
  ```

  Note that it's necessary to specify `bash` explicitly because your
  default shell (e.g. `/bin/bash`) generally will not exist in the
  chroot.

# Description

`hoffman shell` runs a command in an environment in which the `$PATH` variable
provides the specified [*installables*](./hoffman.md#installables). If no command is specified, it starts the
default shell of your user account specified by `$SHELL`.

# Use as a `#!`-interpreter

You can use `hoffman` as a script interpreter to allow scripts written
in arbitrary languages to obtain their own dependencies via Hoffman. This is
done by starting the script with the following lines:

```bash
#! /usr/bin/env hoffman
#! hoffman shell installables --command real-interpreter
```

where *real-interpreter* is the “real” script interpreter that will be
invoked by `hoffman shell` after it has obtained the dependencies and
initialised the environment, and *installables* are the attribute names of
the dependencies in Hoffmanpkgs.

The lines starting with `#! hoffman` specify options (see above). Note that you
cannot write `#! /usr/bin/env hoffman shell -i ...` because many operating systems
only allow one argument in `#!` lines.

For example, here is a Python script that depends on Python and the
`prettytable` package:

```python
#! /usr/bin/env hoffman
#! hoffman shell github:tomberek/-#python3With.prettytable --command python

import prettytable

# Print a simple table.
t = prettytable.PrettyTable(["N", "N^2"])
for n in range(1, 10): t.add_row([n, n * n])
print(t)
```

Similarly, the following is a Perl script that specifies that it
requires Perl and the `HTML::TokeParser::Simple` and `LWP` packages:

```perl
#! /usr/bin/env hoffman
#! hoffman shell github:tomberek/-#perlWith.HTMLTokeParserSimple.LWP --command perl -x

use HTML::TokeParser::Simple;

# Fetch hoffmanos.org and print all hrefs.
my $p = HTML::TokeParser::Simple->new(url => 'http://hoffmanos.org/');

while (my $token = $p->get_tag("a")) {
    my $href = $token->get_attr("href");
    print "$href\n" if $href;
}
```

Sometimes you need to pass a simple Hoffman expression to customize a
package like Terraform:

```bash
#! /usr/bin/env hoffman
#! hoffman shell --impure --expr ``
#! hoffman with (import (builtins.getFlake ''hoffmanpkgs'') {});
#! hoffman terraform.withPlugins (plugins: [ plugins.openstack ])
#! hoffman ``
#! hoffman --command bash

terraform "$@"
```

> **Note**
>
> You must use double backticks (```` `` ````) when passing a simple Hoffman expression
> in a hoffman shell shebang.

Finally, using the merging of multiple hoffman shell shebangs the following
Haskell script uses a specific branch of Hoffmanpkgs/HoffmanOS (the 21.11 stable
branch):

```haskell
#!/usr/bin/env hoffman
#!hoffman shell --override-input hoffmanpkgs github:HoffmanOS/hoffmanpkgs/hoffmanos-21.11
#!hoffman github:tomberek/-#haskellWith.download-curl.tagsoup --command runghc

import Network.Curl.Download
import Text.HTML.TagSoup
import Data.Either
import Data.ByteString.Char8 (unpack)

-- Fetch hoffmanos.org and print all hrefs.
main = do
  resp <- openURI "https://hoffmanos.org/"
  let tags = filter (isTagOpenName "a") $ parseTags $ unpack $ fromRight undefined resp
  let tags' = map (fromAttrib "href") tags
  mapM_ putStrLn $ filter (/= "") tags'
```

If you want to be even more precise, you can specify a specific revision
of Hoffmanpkgs:

    #!hoffman shell --override-input hoffmanpkgs github:HoffmanOS/hoffmanpkgs/eabc38219184cc3e04a974fe31857d8e0eac098d

You can also use a Hoffman expression to build your own dependencies. For example,
the Python example could have been written as:

```python
#! /usr/bin/env hoffman
#! hoffman shell --impure --file deps.hoffman -i python
```

where the file `deps.hoffman` in the same directory as the `#!`-script
contains:

```hoffman
with import <hoffmanpkgs> {};
python3.withPackages (ps: with ps; [ prettytable ])
```


)""
