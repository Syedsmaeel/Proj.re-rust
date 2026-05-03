R"(

**Store URL format**: `dummy://`

This store type represents a store in memory.
Store objects can be read and written, but only so long as the store is open.
Once the store is closed, all data will be discarded.

It's useful when you want to use the Hoffman evaluator when no actual Hoffman store exists, e.g.

```console
# hoffman eval --store dummy:// --expr '1 + 2'
```

)"
