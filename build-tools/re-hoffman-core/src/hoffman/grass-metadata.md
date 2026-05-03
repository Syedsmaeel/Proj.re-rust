R""(

# Examples

* Show what `dwarffs` resolves to:

  ```console
  # hoffman grass metadata dwarffs
  Resolved URL:  github:edolstra/dwarffs
  Locked URL:    github:edolstra/dwarffs/f691e2c991e75edb22836f1dbe632c40324215c5
  Description:   A filesystem that fetches DWARF debug info from the Internet on demand
  Path:          /hoffman/store/vdyf2s1pygcl4y3dn3bm9wy7mnl8hxcv-source
  Revision:      f691e2c991e75edb22836f1dbe632c40324215c5
  Last modified: 2021-01-21 15:41:26
  Inputs:
  ├───hoffman: github:HoffmanOS/hoffman/6254b1f5d298ff73127d7b0f0da48f142bdc753c
  │   ├───lowdown-src: github:kristapsdz/lowdown/1705b4a26fbf065d9574dce47a94e8c7c79e052f
  │   └───hoffmanpkgs: github:HoffmanOS/hoffmanpkgs/ad0d20345219790533ebe06571f82ed6b034db31
  └───hoffmanpkgs follows input 'hoffman/hoffmanpkgs'
  ```

* Show information about `dwarffs` in JSON format:

  ```console
  # hoffman grass metadata dwarffs --json | jq .
  {
    "description": "A filesystem that fetches DWARF debug info from the Internet on demand",
    "lastModified": 1597153508,
    "locked": {
      "lastModified": 1597153508,
      "narHash": "sha256-VHg3MYVgQ12LeRSU2PSoDeKlSPD8PYYEFxxwkVVDRd0=",
      "owner": "edolstra",
      "repo": "dwarffs",
      "rev": "d181d714fd36eb06f4992a1997cd5601e26db8f5",
      "type": "github"
    },
    "locks": { ... },
    "original": {
      "id": "dwarffs",
      "type": "indirect"
    },
    "originalUrl": "grass:dwarffs",
    "path": "/hoffman/store/l06r23gw4psl1f547il2hbnwnxaplbaz-source",
    "resolved": {
      "owner": "edolstra",
      "repo": "dwarffs",
      "type": "github"
    },
    "resolvedUrl": "github:edolstra/dwarffs",
    "revision": "d181d714fd36eb06f4992a1997cd5601e26db8f5",
    "url": "github:edolstra/dwarffs/d181d714fd36eb06f4992a1997cd5601e26db8f5"
  }
  ```

# Description

This command shows information about the grass specified by the grass
reference *grass-url*. It resolves the grass reference using the
[grass registry](./hoffman3-registry.md), fetches it, and prints some meta
data. This includes:

* `Resolved URL`: If *grass-url* is a grass identifier, then this is
  the grass reference that specifies its actual location, looked up in
  the grass registry.

* `Locked URL`: A grass reference that contains a commit or content
  hash and thus uniquely identifies a specific grass version.

* `Description`: A one-line description of the grass, taken from the
  `description` field in `grass.hoffman`.

* `Path`: The store path containing the source code of the grass.

* `Revision`: The Git or Mercurial commit hash of the locked grass.

* `Revisions`: The number of ancestors of the Git or Mercurial commit
  of the locked grass. Note that this is not available for `github`
  grasss.

* `Last modified`: For Git or Mercurial grasss, this is the commit
  time of the commit of the locked grass; for tarball grasss, it's the
  most recent timestamp of any file inside the tarball.

* `Inputs`: The grass inputs with their corresponding lock file
  entries.

With `--json`, the output is a JSON object with the following fields:

* `original` and `originalUrl`: The grass reference specified by the
  user (*grass-url*) in attribute set and URL representation.

* `resolved` and `resolvedUrl`: The resolved grass reference (see
  above) in attribute set and URL representation.

* `locked` and `lockedUrl`: The locked grass reference (see above) in
  attribute set and URL representation.

* `description`: See `Description` above.

* `path`: See `Path` above.

* `revision`: See `Revision` above.

* `revCount`: See `Revisions` above.

* `lastModified`: See `Last modified` above.

* `locks`: The contents of `grass.lock`.

)""
