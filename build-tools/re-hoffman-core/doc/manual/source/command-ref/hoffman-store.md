# Name

`hoffman-store` - manipulate or query the Hoffman store

# Synopsis

`hoffman-store` *operation* [*options…*] [*arguments…*]
  [`--option` *name* *value*]
  [`--add-root` *path*]

# Description

The command `hoffman-store` performs primitive operations on the Hoffman store.
You generally do not need to run this command manually.

`hoffman-store` takes exactly one *operation* flag which indicates the subcommand to be performed. The following operations are available:

- [`--realise`](./hoffman-store/realise.md)
- [`--serve`](./hoffman-store/serve.md)
- [`--gc`](./hoffman-store/gc.md)
- [`--delete`](./hoffman-store/delete.md)
- [`--query`](./hoffman-store/query.md)
- [`--add`](./hoffman-store/add.md)
- [`--add-fixed`](./hoffman-store/add-fixed.md)
- [`--verify`](./hoffman-store/verify.md)
- [`--verify-path`](./hoffman-store/verify-path.md)
- [`--repair-path`](./hoffman-store/repair-path.md)
- [`--dump`](./hoffman-store/dump.md)
- [`--restore`](./hoffman-store/restore.md)
- [`--export`](./hoffman-store/export.md)
- [`--import`](./hoffman-store/import.md)
- [`--optimise`](./hoffman-store/optimise.md)
- [`--read-log`](./hoffman-store/read-log.md)
- [`--dump-db`](./hoffman-store/dump-db.md)
- [`--load-db`](./hoffman-store/load-db.md)
- [`--print-env`](./hoffman-store/print-env.md)
- [`--generate-binary-cache-key`](./hoffman-store/generate-binary-cache-key.md)

These pages can be viewed offline:

- `man hoffman-store-<operation>`.

  Example: `man hoffman-store-realise`

- `hoffman-store --help --<operation>`

  Example: `hoffman-store --help --realise`
