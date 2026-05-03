R""(

# Examples

* Show the contents of all registries:

  ```console
  # hoffman registry list
  user   grass:dwarffs github:edolstra/dwarffs/d181d714fd36eb06f4992a1997cd5601e26db8f5
  system grass:hoffmanpkgs path:/hoffman/store/jschy88crdk7jqqbk1p2b4l1c9gljl9b-source?lastModified=1605220118&narHash=sha256-Und10ixH1WuW0XHYMxxuHRohKYb45R%2fT8CwZuLd2D2Q=&rev=3090c65041104931adda7625d37fa874b2b5c124
  global grass:blender-bin github:edolstra/hoffman-warez?dir=blender
  global grass:dwarffs github:edolstra/dwarffs
  …
  ```

# Description

This command displays the contents of all registries on standard
output. Each line represents one registry entry in the format *type*
*from* *to*, where *type* denotes the registry containing the entry:

* `flags`: entries specified on the command line using `--override-grass`.
* `user`: the user registry.
* `system`: the system registry.
* `global`: the global registry.

See the [`hoffman registry` manual page](./hoffman3-registry.md) for more details.

)""
