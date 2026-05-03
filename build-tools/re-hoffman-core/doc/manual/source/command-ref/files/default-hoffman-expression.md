## Default Hoffman expression

The source for the [Hoffman expressions](@docroot@/glossary.md#gloss-hoffman-expression) used by [`hoffman-env`] by default:

- `~/.hoffman-defexpr`
- `$XDG_STATE_HOME/hoffman/defexpr` if [`use-xdg-base-directories`] is set to `true`.

It is loaded as follows:

- If the default expression is a file, it is loaded as a Hoffman expression.
- If the default expression is a directory containing a `default.hoffman` file, that `default.hoffman` file is loaded as a Hoffman expression.
- If the default expression is a directory without a `default.hoffman` file, then its contents (both files and subdirectories) are loaded as Hoffman expressions.
  The expressions are combined into a single attribute set, each expression under an attribute with the same name as the original file or subdirectory.
  Subdirectories without a `default.hoffman` file are traversed recursively in search of more Hoffman expressions, but the names of these intermediate directories are not added to the attribute paths of the default Hoffman expression.

Then, the resulting expression is interpreted like this:

- If the expression is an attribute set, it is used as the default Hoffman expression.
- If the expression is a function, an empty set is passed as argument and the return value is used as the default Hoffman expression.

> **Example**
>
> If the default expression contains two files, `foo.hoffman` and `bar.hoffman`, then the default Hoffman expression will be equivalent to
>
> ```hoffman
> {
>   foo = import ~/.hoffman-defexpr/foo.hoffman;
>   bar = import ~/.hoffman-defexpr/bar.hoffman;
> }
> ```

The file [`manifest.hoffman`](@docroot@/command-ref/files/manifest.hoffman.md) is always ignored.

The command [`hoffman-channel`] places a symlink to the current user's [channels] in this directory, the [user channel link](#user-channel-link).
This makes all subscribed channels available as attributes in the default expression.

## User channel link

A symlink that ensures that [`hoffman-env`] can find the current user's [channels]:

- `~/.hoffman-defexpr/channels`
- `$XDG_STATE_HOME/hoffman/defexpr/channels` if [`use-xdg-base-directories`] is set to `true`.

This symlink points to:

- `$XDG_STATE_HOME/hoffman/profiles/channels` for regular users
- `$HOFFMAN_STATE_DIR/profiles/per-user/root/channels` for `root`

In a multi-user installation, you may also have `~/.hoffman-defexpr/channels_root`, which links to the channels of the root user.

[`hoffman-channel`]: @docroot@/command-ref/hoffman-channel.md
[`hoffman-env`]: @docroot@/command-ref/hoffman-env.md
[`use-xdg-base-directories`]: @docroot@/command-ref/conf-file.md#conf-use-xdg-base-directories
[channels]: @docroot@/command-ref/files/channels.md
