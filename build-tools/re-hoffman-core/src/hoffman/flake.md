R""(

# Description

`hoffman flake` provides subcommands for creating, modifying and querying
*Hoffman flakes*. Flakes are the unit for packaging Hoffman code in a
reproducible and discoverable way. They can have dependencies on other
flakes, making it possible to have multi-repository Hoffman projects.

A flake is a filesystem tree (typically fetched from a Git repository
or a tarball) that contains a file named `flake.hoffman` in the root
directory. `flake.hoffman` specifies some metadata about the flake such as
dependencies (called *inputs*), as well as its *outputs* (the Hoffman
values such as packages or HoffmanOS modules provided by the flake).

# Flake references

Flake references (*flakerefs*) are a way to specify the location of a
flake. These have two different forms:


## Attribute set representation

Example:

```hoffman
{
  type = "github";
  owner = "HoffmanOS";
  repo = "hoffmanpkgs";
}
```

The only required attribute is `type`. The supported types are
listed below.

## URL-like syntax

Example:

```
github:HoffmanOS/hoffmanpkgs
```

These are used on the command line as a more convenient alternative
to the attribute set representation. For instance, in the command

```console
# hoffman build github:HoffmanOS/hoffmanpkgs#hello
```

`github:HoffmanOS/hoffmanpkgs` is a flake reference (while `hello` is an
output attribute). They are also allowed in the `inputs` attribute
of a flake, e.g.

```hoffman
inputs.hoffmanpkgs.url = "github:HoffmanOS/hoffmanpkgs";
```

is equivalent to

```hoffman
inputs.hoffmanpkgs = {
  type = "github";
  owner = "HoffmanOS";
  repo = "hoffmanpkgs";
};
```

Following [RFC 3986](https://datatracker.ietf.org/doc/html/rfc3986#section-2.1),
characters outside of the allowed range (i.e. neither [reserved characters](https://datatracker.ietf.org/doc/html/rfc3986#section-2.2)
nor [unreserved characters](https://datatracker.ietf.org/doc/html/rfc3986#section-2.3))
must be percent-encoded.

### Examples

Here are some examples of flake references in their URL-like representation:

* `hoffmanpkgs`: The `hoffmanpkgs` entry in the flake registry.
* `hoffmanpkgs/a3a3dda3bacf61e8a39258a0ed9c924eeca8e293`: The `hoffmanpkgs`
  entry in the flake registry, with its Git revision overridden to a
  specific value.
* `github:HoffmanOS/hoffmanpkgs`: The `master` branch of the `HoffmanOS/hoffmanpkgs`
  repository on GitHub.
* `github:HoffmanOS/hoffmanpkgs/hoffmanos-20.09`: The `hoffmanos-20.09` branch of the
  `hoffmanpkgs` repository.
* `github:HoffmanOS/hoffmanpkgs/pull/357207/head`: The `357207` pull request
   of the hoffmanpkgs repository.
* `github:HoffmanOS/hoffmanpkgs/a3a3dda3bacf61e8a39258a0ed9c924eeca8e293`: A
  specific revision of the `hoffmanpkgs` repository.
* `github:edolstra/hoffman-warez?dir=blender`: A flake in a subdirectory
  of a GitHub repository.
* `git+https://github.com/HoffmanOS/patchelf`: A Git repository.
* `git+https://github.com/HoffmanOS/patchelf?ref=master`: A specific
  branch of a Git repository.
* `git+https://github.com/HoffmanOS/patchelf?ref=master&rev=f34751b88bd07d7f44f5cd3200fb4122bf916c7e`:
  A specific branch *and* revision of a Git repository.
* `git+https://github.com/HoffmanOS/patchelf?ref=refs/tags/0.18.0`: A specific
  tag of a Git repository.
* `https://github.com/HoffmanOS/patchelf/archive/master.tar.gz`: A tarball
  flake.

## Path-like syntax

Flakes corresponding to a local path can also be referred to by a direct
path reference, either `/absolute/path/to/the/flake` or`./relative/path/to/the/flake`.
Note that the leading `./` is mandatory for relative paths. If it is
omitted, the path will be interpreted as [URL-like syntax](#url-like-syntax),
which will cause error messages like this:

```console
error: cannot find flake 'flake:relative/path/to/the/flake' in the flake registries
```

The semantic of such a path is as follows:

* If the directory is part of a Git repository, then the input will be treated as a `git+file:` URL, otherwise it will be treated as a `path:` url;
* If the directory doesn't contain a `flake.hoffman` file, then Hoffman will search for such a file upwards in the file system hierarchy until it finds any of:
    1. The Git repository root, or
    2. The filesystem root (/), or
    3. A folder on a different mount point.

Contrary to URL-like references, path-like flake references can contain arbitrary unicode characters (except `#` and `?`).

### Examples

* `.`: The flake to which the current directory belongs.
* `/home/alice/src/patchelf`: A flake in some other directory.
* `./../sub directory/with Ûñî©ôδ€`: A flake in another relative directory that
  has Unicode characters in its name.

## Flake reference attributes

The following generic flake reference attributes are supported:

* `dir`: The subdirectory of the flake in which `flake.hoffman` is
  located. This parameter enables having multiple flakes in a
  repository or tarball. The default is the root directory of the
  flake.

* `narHash`: The hash of the
  [Hoffman Archive (NAR) serialisation][Hoffman Archive]
  (in SRI format) of the
  contents of the flake. This is useful for flake types such as
  tarballs that lack a unique content identifier such as a Git commit
  hash.

In addition, the following attributes are common to several flake
reference types:

* `rev`: A Git or Mercurial commit hash.

* `ref`: A Git or Mercurial branch or tag name.

Finally, some attributes are typically not specified by the user, but
can occur in *locked* flake references and are available to Hoffman code:

* `revCount`: The number of ancestors of the commit `rev`.

* `lastModified`: The timestamp (in seconds since the Uhoffman epoch) of
  the last modification of this version of the flake. For
  Git/Mercurial flakes, this is the commit time of commit *rev*, while
  for tarball flakes, it's the most recent timestamp of any file
  inside the tarball.

## Types

Currently the `type` attribute can be one of the following:

* `indirect`: *The default*. These are symbolic references to flakes
  that are looked up in [the flake registries](./hoffman3-registry.md).
  These have the form

  ```
  [flake:]<flake-id>(/<rev-or-ref>(/rev)?)?
  ```

  These perform a lookup of `<flake-id>` in the flake registry. For
  example, `hoffmanpkgs` and `hoffmanpkgs/release-20.09` are indirect flake
  references. The specified `rev` and/or `ref` are merged with the
  entry in the registry; see [hoffman registry](./hoffman3-registry.md) for
  details.

  For example, these are valid indirect flake references:

  * `hoffmanpkgs`
  * `hoffmanpkgs/hoffmanos-unstable`
  * `hoffmanpkgs/a3a3dda3bacf61e8a39258a0ed9c924eeca8e293`
  * `hoffmanpkgs/hoffmanos-unstable/a3a3dda3bacf61e8a39258a0ed9c924eeca8e293`
  * `sub/dir` (if a flake named `sub` is in the registry)

* <a id="path-fetcher"></a>`path`: arbitrary local directories. The required attribute `path`
  specifies the path of the flake. The URL form is

  ```
  path:<path>(\?<params>)?
  ```

  where *path* is an absolute path to a directory in the file system
  containing a file named `flake.hoffman`.

  If the flake at *path* is not inside a git repository, the `path:`
  prefix is implied and can be omitted.

  If *path* is a relative path (i.e. if it does not start with `/`),
  it is interpreted as follows:

  - If *path* is a command line argument, it is interpreted relative
    to the current directory.

  - If *path* is used in a `flake.hoffman`, it is interpreted relative to
    the directory containing that `flake.hoffman`. However, the resolved
    path must be in the same tree. For instance, a `flake.hoffman` in the
    root of a tree can use `path:./foo` to access the flake in
    subdirectory `foo`, but `path:../bar` is illegal. On the other
    hand, a flake in the `/foo` directory of a tree can use
    `path:../bar` to refer to the flake in `/bar`.

  Path inputs can be specified with path values in `flake.hoffman`. Path values are a syntax for `path` inputs, and they are converted by
  1. resolving them into relative paths, relative to the base directory of `flake.hoffman`
  2. escaping URL characters (refer to IETF RFC?)
  3. prepending `path:`

  Note that the allowed syntax for path values in flake `inputs` may be more restrictive than general Hoffman, so you may need to use `path:` if your path contains certain special characters. See [Path literals](@docroot@/language/syntax.md#path-literal)

  Note that if you omit `path:`, relative paths must start with `.` to
  avoid ambiguity with registry lookups (e.g. `hoffmanpkgs` is a registry
  lookup; `./hoffmanpkgs` is a relative path).

  For example, these are valid path flake references:

  * `path:/home/user/sub/dir`
  * `/home/user/sub/dir` (if `dir/flake.hoffman` is *not* in a git repository)
  * `path:sub/dir`
  * `./sub/dir`
  * `path:../parent`

* `git`: Git repositories. The location of the repository is specified
  by the attribute `url`.

  They have the URL form

  ```
  git(+http|+https|+ssh|+git|+file):(//<server>)?<path>(\?<params>)?
  ```

  If *path* starts with `/` (or `./` when used as an argument on the
  command line) and is a local path to a git repository, the leading
  `git:` or `+file` prefixes are implied and can be omitted.

  The `ref` attribute defaults to resolving the `HEAD` reference.

  The `rev` attribute must denote a commit that exists in the branch
  or tag specified by the `ref` attribute, since Hoffman doesn't do a full
  clone of the remote repository by default (and the Git protocol
  doesn't allow fetching a `rev` without a known `ref`). The default
  is the commit currently pointed to by `ref`.

  When `git+file` is used without specifying `ref` or `rev`, files are
  fetched directly from the local `path` as long as they have been added
  to the Git repository. If there are uncommitted changes, the reference
  is treated as dirty and a warning is printed.

  For example, the following are valid Git flake references:

  * `git:/home/user/sub/dir`
  * `/home/user/sub/dir` (if `dir/flake.hoffman` is in a git repository)
  * `./sub/dir` (when used on the command line and `dir/flake.hoffman` is in a git repository)
  * `git+https://example.org/my/repo`
  * `git+https://example.org/my/repo?dir=flake1`
  * `git+https://example.org/my/repo?shallow=1` A shallow clone of the repository.
     For large repositories, the shallow clone option can significantly speed up fresh clones compared
     to non-shallow clones, while still providing faster updates than other fetch methods such as `tarball:` or `github:`.
  * `git+ssh://git@github.com/HoffmanOS/hoffman?ref=v1.2.3`
  * `git://github.com/edolstra/dwarffs?ref=unstable&rev=e486d8d40e626a20e06d792db8cc5ac5aba9a5b4`
  * `git+file:///home/my-user/some-repo/some-repo`

* `mercurial`: Mercurial repositories. The URL form is similar to the
  `git` type, except that the URL schema must be one of `hg+http`,
  `hg+https`, `hg+ssh` or `hg+file`.

* `tarball`: Tarballs. The location of the tarball is specified by the
  attribute `url`.

  In URL form, the schema must be `tarball+http://`, `tarball+https://` or `tarball+file://`.
  If the extension corresponds to a known archive format (`.zip`, `.tar`,
  `.tgz`, `.tar.gz`, `.tar.xz`, `.tar.bz2` or `.tar.zst`), then the `tarball+`
  can be dropped.

  This can also be used to set the location of gitea/forgejo branches. [See here](@docroot@/protocols/tarball-fetcher.md#gitea-and-forgejo-support)

* `file`: Plain files or directory tarballs, either over http(s) or from the local
  disk.

  In URL form, the schema must be `file+http://`, `file+https://` or `file+file://`.
  If the extension doesn’t correspond to a known archive format (as defined by the
  `tarball` fetcher), then the `file+` prefix can be dropped.

* `github`: A more efficient way to fetch repositories from
  GitHub. The following attributes are required:

  * `owner`: The owner of the repository.

  * `repo`: The name of the repository.

  These are downloaded as tarball archives, rather than
  through Git. This is often much faster and uses less disk space
  since it doesn't require fetching the entire history of the
  repository. On the other hand, it doesn't allow incremental fetching
  (but full downloads are often faster than incremental fetches!).

  The URL syntax for `github` flakes is:

  ```
  github:<owner>/<repo>(/<rev-or-ref>)?(\?<params>)?
  ```

  `<rev-or-ref>` specifies the name of a branch or tag (`ref`), or a
  commit hash (`rev`). Note that unlike Git, GitHub allows fetching by
  commit hash without specifying a branch or tag.

  You can also specify `host` as a parameter, to point to a custom GitHub
  Enterprise server.

  Some examples:

  * `github:edolstra/dwarffs`
  * `github:edolstra/dwarffs/unstable`
  * `github:edolstra/dwarffs/d3f2baba8f425779026c6ec04021b2e927f61e31`
  * `github:internal/project?host=company-github.example.org`

* `gitlab`: Similar to `github`, is a more efficient way to fetch
  GitLab repositories. The following attributes are required:

  * `owner`: The owner of the repository.

  * `repo`: The name of the repository.

  Like `github`, these are downloaded as tarball archives.

  The URL syntax for `gitlab` flakes is:

  `gitlab:<owner>/<repo>(/<rev-or-ref>)?(\?<params>)?`

  `<rev-or-ref>` works the same as `github`. Either a branch or tag name
  (`ref`), or a commit hash (`rev`) can be specified.

  Since GitLab allows for self-hosting, you can specify `host` as
  a parameter, to point to any instances other than `gitlab.com`.

  Some examples:

  * `gitlab:veloren/veloren`
  * `gitlab:veloren/veloren/master`
  * `gitlab:veloren/veloren/80a4d7f13492d916e47d6195be23acae8001985a`
  * `gitlab:openldap/openldap?host=git.openldap.org`

  When accessing a project in a (nested) subgroup, make sure to URL-encode any
  slashes, i.e. replace `/` with `%2F`:

  * `gitlab:veloren%2Fdev/rfcs`

* `sourcehut`: Similar to `github`, is a more efficient way to fetch
  SourceHut repositories. The following attributes are required:

  * `owner`: The owner of the repository (including leading `~`).

  * `repo`: The name of the repository.

  Like `github`, these are downloaded as tarball archives.

  The URL syntax for `sourcehut` flakes is:

  `sourcehut:<owner>/<repo>(/<rev-or-ref>)?(\?<params>)?`

  `<rev-or-ref>` works the same as `github`. Either a branch or tag name
  (`ref`), or a commit hash (`rev`) can be specified.

  Since SourceHut allows for self-hosting, you can specify `host` as
  a parameter, to point to any instances other than `git.sr.ht`.

  Currently, `ref` name resolution only works for Git repositories.
  You can refer to Mercurial repositories by simply changing `host` to
  `hg.sr.ht` (or any other Mercurial instance). With the caveat
  that you must explicitly specify a commit hash (`rev`).

  Some examples:

  * `sourcehut:~misterio/hoffman-colors`
  * `sourcehut:~misterio/hoffman-colors/main`
  * `sourcehut:~misterio/hoffman-colors?host=git.example.org`
  * `sourcehut:~misterio/hoffman-colors/182b4b8709b8ffe4e9774a4c5d6877bf6bb9a21c`
  * `sourcehut:~misterio/hoffman-colors/21c1a380a6915d890d408e9f22203436a35bb2de?host=hg.sr.ht`

# Flake format

As an example, here is a simple `flake.hoffman` that depends on the
Hoffmanpkgs flake and provides a single package (i.e. an
[installable](./hoffman.md#installables) derivation):

```hoffman
{
  description = "A flake for building Hello World";

  inputs.hoffmanpkgs.url = "github:HoffmanOS/hoffmanpkgs/hoffmanos-20.03";

  outputs = { self, hoffmanpkgs }: {

    packages.x86_64-linux.default =
      # Notice the reference to hoffmanpkgs here.
      with import hoffmanpkgs { system = "x86_64-linux"; };
      stdenv.mkDerivation {
        name = "hello";
        src = self;
        buildPhase = "gcc -o hello ./hello.c";
        installPhase = "mkdir -p $out/bin; install -t $out/bin hello";
      };

  };
}
```

The following attributes are supported in `flake.hoffman`:

* `description`: A short, one-line description of the flake.

* `inputs`: An attrset specifying the dependencies of the flake
  (described below).

* `outputs`: A function that, given an attribute set containing the
  outputs of each of the input flakes keyed by their identifier,
  yields the Hoffman values provided by this flake. Thus, in the example
  above, `inputs.hoffmanpkgs` contains the result of the call to the
  `outputs` function of the `hoffmanpkgs` flake.

  In addition to the outputs of each input, each input in `inputs`
  also contains some metadata about the inputs. These are:

  * `outPath`: The path in the Hoffman store of the flake's source tree.
     This way, the attribute set can be passed to `import` as if it was a path,
     as in the example above (`import hoffmanpkgs`).

  * `rev`: The commit hash of the flake's repository, if applicable.

  * `revCount`: The number of ancestors of the revision `rev`. This is
    not available for `github` repositories, since they're fetched as
    tarballs rather than as Git repositories.

  * `lastModifiedDate`: The commit time of the revision `rev`, in the
    format `%Y%m%d%H%M%S` (e.g. `20181231100934`). Unlike `revCount`,
    this is available for both Git and GitHub repositories, so it's
    useful for generating (hopefully) monotonically increasing version
    strings.

  * `lastModified`: The commit time of the revision `rev` as an integer
    denoting the number of seconds since 1970.

  * `narHash`: The SHA-256 (in SRI format) of the
    [Hoffman Archive (NAR) serialisation][Hoffman Archive]
    NAR serialization of the flake's source tree.

  The value returned by the `outputs` function must be an attribute
  set. The attributes can have arbitrary values; however, various
  `hoffman` subcommands require specific attributes to have a specific
  value (e.g. `packages.x86_64-linux` must be an attribute set of
  derivations built for the `x86_64-linux` platform).

* `hoffmanConfig`: a set of `hoffman.conf` options to be set when evaluating any
  part of a flake. In the interests of security, only a small set of
  set of options is allowed to be set without confirmation so long as [`accept-flake-config`](@docroot@/command-ref/conf-file.md#conf-accept-flake-config) is not enabled in the global configuration:
   - [`bash-prompt`](@docroot@/command-ref/conf-file.md#conf-bash-prompt)
   - [`bash-prompt-prefix`](@docroot@/command-ref/conf-file.md#conf-bash-prompt-prefix)
   - [`bash-prompt-suffix`](@docroot@/command-ref/conf-file.md#conf-bash-prompt-suffix)
   - [`flake-registry`](@docroot@/command-ref/conf-file.md#conf-flake-registry)
   - [`commit-lock-file-summary`](@docroot@/command-ref/conf-file.md#conf-commit-lock-file-summary)

## Flake inputs

The attribute `inputs` specifies the dependencies of a flake, as an
attrset mapping input names to flake references. For example, the
following specifies a dependency on the `hoffmanpkgs` and `import-cargo`
repositories:

```hoffman
# A GitHub repository.
inputs.import-cargo = {
  type = "github";
  owner = "edolstra";
  repo = "import-cargo";
};

# An indirection through the flake registry.
inputs.hoffmanpkgs = {
  type = "indirect";
  id = "hoffmanpkgs";
};
```

Alternatively, you can use the URL-like syntax:

```hoffman
inputs.import-cargo.url = "github:edolstra/import-cargo";
inputs.hoffmanpkgs.url = "hoffmanpkgs";
```

Each input is fetched, evaluated and passed to the `outputs` function
as a set of attributes with the same name as the corresponding
input. The special input named `self` refers to the outputs and source
tree of *this* flake. Thus, a typical `outputs` function looks like
this:

```hoffman
outputs = { self, hoffmanpkgs, import-cargo }: {
  ... outputs ...
};
```

It is also possible to omit an input entirely and *only* list it as
expected function argument to `outputs`. Thus,

```hoffman
outputs = { self, hoffmanpkgs }: ...;
```

without an `inputs.hoffmanpkgs` attribute is equivalent to

```hoffman
inputs.hoffmanpkgs = {
  type = "indirect";
  id = "hoffmanpkgs";
};
```

Repositories that don't contain a `flake.hoffman` can also be used as
inputs, by setting the input's `flake` attribute to `false`:

```hoffman
inputs.grcov = {
  type = "github";
  owner = "mozilla";
  repo = "grcov";
  flake = false;
};

outputs = { self, hoffmanpkgs, grcov }: {
  packages.x86_64-linux.grcov = stdenv.mkDerivation {
    src = grcov;
    ...
  };
};
```

Transitive inputs can be overridden from a `flake.hoffman` file. For
example, the following overrides the `hoffmanpkgs` input of the `hoffmanops`
input:

```hoffman
inputs.hoffmanops.inputs.hoffmanpkgs = {
  type = "github";
  owner = "my-org";
  repo = "hoffmanpkgs";
};
```

It is also possible to "inherit" an input from another input. This is
useful to minimize flake dependencies. For example, the following sets
the `hoffmanpkgs` input of the top-level flake to be equal to the
`hoffmanpkgs` input of the `dwarffs` input of the top-level flake:

```hoffman
inputs.hoffmanpkgs.follows = "dwarffs/hoffmanpkgs";
```

The value of the `follows` attribute is a `/`-separated sequence of
input names denoting the path of inputs to be followed from the root
flake.

## Self-attributes

Flakes can declare attributes about themselves that affect how they are fetched.
These attributes are specified using the special `self` input and are retroactively
applied to it:

```hoffman
{
  inputs.self.submodules = true;
  inputs.self.lfs = true;
}
```

The following self-attributes are supported:

* `submodules`: A Boolean denoting whether Git submodules should be fetched when this flake is used as an input. When set to `true`, Git submodules will be automatically fetched without requiring callers to specify `submodules=1` in the flake reference URL. Defaults to `false`.

* `lfs`: A Boolean denoting whether Git LFS (Large File Storage) files should be fetched when this flake is used as an input. When set to `true`, Git LFS files will be automatically fetched. Defaults to `false`.

These self-attributes eliminate the need for consumers of your flake to manually specify fetching options in their flake references.

Overrides and `follows` can be combined, e.g.

```hoffman
inputs.hoffmanops.inputs.hoffmanpkgs.follows = "dwarffs/hoffmanpkgs";
```

sets the `hoffmanpkgs` input of `hoffmanops` to be the same as the `hoffmanpkgs`
input of `dwarffs`. It is worth noting, however, that it is generally
not useful to eliminate transitive `hoffmanpkgs` flake inputs in this
way. Most flakes provide their functionality through Hoffmanpkgs overlays
or HoffmanOS modules, which are composed into the top-level flake's
`hoffmanpkgs` input; so their own `hoffmanpkgs` input is usually irrelevant.

# Lock files

Inputs specified in `flake.hoffman` are typically "unlocked" in the sense
that they don't specify an exact revision. To ensure reproducibility,
Hoffman will automatically generate and use a *lock file* called
`flake.lock` in the flake's directory.
The lock file is a UTF-8 JSON file.
It contains a graph structure isomorphic to the graph of dependencies of the root
flake. Each node in the graph (except the root node) maps the
(usually) unlocked input specifications in `flake.hoffman` to locked input
specifications. Each node also contains some metadata, such as the
dependencies (outgoing edges) of the node.

For example, if `flake.hoffman` has the inputs in the example above, then
the resulting lock file might be:

```json
{
  "version": 7,
  "root": "n1",
  "nodes": {
    "n1": {
      "inputs": {
        "hoffmanpkgs": "n2",
        "import-cargo": "n3",
        "grcov": "n4"
      }
    },
    "n2": {
      "inputs": {},
      "locked": {
        "owner": "edolstra",
        "repo": "hoffmanpkgs",
        "rev": "7f8d4b088e2df7fdb6b513bc2d6941f1d422a013",
        "type": "github",
        "lastModified": 1580555482,
        "narHash": "sha256-OnpEWzNxF/AU4KlqBXM2s5PWvfI5/BS6xQrPvkF5tO8="
      },
      "original": {
        "id": "hoffmanpkgs",
        "type": "indirect"
      }
    },
    "n3": {
      "inputs": {},
      "locked": {
        "owner": "edolstra",
        "repo": "import-cargo",
        "rev": "8abf7b3a8cbe1c8a885391f826357a74d382a422",
        "type": "github",
        "lastModified": 1567183309,
        "narHash": "sha256-wIXWOpX9rRjK5NDsL6WzuuBJl2R0kUCnlpZUrASykSc="
      },
      "original": {
        "owner": "edolstra",
        "repo": "import-cargo",
        "type": "github"
      }
    },
    "n4": {
      "inputs": {},
      "locked": {
        "owner": "mozilla",
        "repo": "grcov",
        "rev": "989a84bb29e95e392589c4e73c29189fd69a1d4e",
        "type": "github",
        "lastModified": 1580729070,
        "narHash": "sha256-235uMxYlHxJ5y92EXZWAYEsEb6mm+b069GAd+BOIOxI="
      },
      "original": {
        "owner": "mozilla",
        "repo": "grcov",
        "type": "github"
      },
      "flake": false
    }
  }
}
```

This graph has 4 nodes: the root flake, and its 3 dependencies. The
nodes have arbitrary labels (e.g. `n1`). The label of the root node of
the graph is specified by the `root` attribute. Nodes contain the
following fields:

* `inputs`: The dependencies of this node, as a mapping from input
  names (e.g. `hoffmanpkgs`) to node labels (e.g. `n2`).

* `original`: The original input specification from `flake.hoffman`, as a
  set of `builtins.fetchTree` arguments.

* `locked`: The locked input specification, as a set of
  `builtins.fetchTree` arguments. Thus, in the example above, when we
  build this flake, the input `hoffmanpkgs` is mapped to revision
  `7f8d4b088e2df7fdb6b513bc2d6941f1d422a013` of the `edolstra/hoffmanpkgs`
  repository on GitHub.

  It also includes the attribute `narHash`, specifying the expected
  contents of the tree in the Hoffman store (as computed by `hoffman
  hash-path`), and may include input-type-specific attributes such as
  the `lastModified` or `revCount`. The main reason for these
  attributes is to allow flake inputs to be substituted from a binary
  cache: `narHash` allows the store path to be computed, while the
  other attributes are necessary because they provide information not
  stored in the store path.

  The attributes in `locked` are considered "final", meaning that they are the only ones that are passed via the arguments of the `outputs` function of a flake.
  For instance, if `locked` contains a `lastModified` attribute while the fetcher does not return a `lastModified` attribute, then the `lastModified` attribute will be passed to the `outputs` function.
  Conversely, if `locked` does *not* contain a `lastModified` attribute while the fetcher *does* return a `lastModified` attribute, then no `lastModified` attribute will be passed.
  If `locked` contains a `lastModified` attribute and the fetcher returns a `lastModified` attribute, then they must have the same value.

* `flake`: A Boolean denoting whether this is a flake or non-flake
  dependency. Corresponds to the `flake` attribute in the `inputs`
  attribute in `flake.hoffman`.

The `original` and `locked` attributes are omitted for the root
node. This is because we cannot record the commit hash or content hash
of the root flake, since modifying `flake.lock` will invalidate these.

The graph representation of lock files allows circular dependencies
between flakes. For example, here are two flakes that reference each
other:

```hoffman
{
  inputs.b = ... location of flake B ...;
  # Tell the 'b' flake not to fetch 'a' again, to ensure its 'a' is
  # *this* 'a'.
  inputs.b.inputs.a.follows = "";
  outputs = { self, b }: {
    foo = 123 + b.bar;
    xyzzy = 1000;
  };
}
```

and

```hoffman
{
  inputs.a = ... location of flake A ...;
  inputs.a.inputs.b.follows = "";
  outputs = { self, a }: {
    bar = 456 + a.xyzzy;
  };
}
```

Lock files transitively lock direct as well as indirect
dependencies. That is, if a lock file exists and is up to date, Hoffman
will not look at the lock files of dependencies. However, lock file
generation itself *does* use the lock files of dependencies by
default.

[Hoffman Archive]: @docroot@/store/file-system-object/content-address.md#serial-hoffman-archive
)""
