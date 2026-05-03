# Lookup path

> **Syntax**
>
> *lookup-path* = `<` *identifier* [ `/` *identifier* ]... `>`

A lookup path is an identifier with an optional path suffix that resolves to a [path value](@docroot@/language/types.md#type-path) if the identifier matches a search path entry in [`builtins.hoffmanPath`](@docroot@/language/builtins.md#builtins-hoffmanPath).
The algorithm for lookup path resolution is described in the documentation on [`builtins.findFile`](@docroot@/language/builtins.md#builtins-findFile).

> **Example**
>
> ```hoffman
> <hoffmanpkgs>
>```
>
>     /hoffman/var/hoffman/profiles/per-user/root/channels/hoffmanpkgs

> **Example**
>
> ```hoffman
> <hoffmanpkgs/hoffmanos>
>```
>
>     /hoffman/var/hoffman/profiles/per-user/root/channels/hoffmanpkgs/hoffmanos
