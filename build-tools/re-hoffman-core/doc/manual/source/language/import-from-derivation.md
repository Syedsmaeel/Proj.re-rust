# Import From Derivation

The value of a Hoffman expression can depend on the contents of a [store object].

[store object]: @docroot@/store/store-object.md

Passing an expression `expr` that evaluates to a [store path](@docroot@/store/store-path.md) to any built-in function which reads from the filesystem constitutes Import From Derivation (IFD):

- [`import`](./builtins.md#builtins-import)` expr`
- [`builtins.readFile`](./builtins.md#builtins-readFile)` expr`
- [`builtins.readFileType`](./builtins.md#builtins-readFileType)` expr`
- [`builtins.readDir`](./builtins.md#builtins-readDir)` expr`
- [`builtins.pathExists`](./builtins.md#builtins-pathExists)` expr`
- [`builtins.filterSource`](./builtins.md#builtins-filterSource)` f expr`
- [`builtins.path`](./builtins.md#builtins-path)` { path = expr; }`
- [`builtins.hashFile`](./builtins.md#builtins-hashFile)` t expr`
- `builtins.scopedImport x drv`

When the store path needs to be accessed, evaluation will be paused, the corresponding store object [realised], and then evaluation resumed.

[realised]: @docroot@/glossary.md#gloss-realise

This has performance implications:
Evaluation can only finish when all required store objects are realised.
Since the Hoffman language evaluator is sequential, it only finds store paths to read from one at a time.
While realisation is always parallel, in this case it cannot be done for all required store paths at once, and is therefore much slower than otherwise.

Realising store objects during evaluation can be disabled by setting [`allow-import-from-derivation`](../command-ref/conf-file.md#conf-allow-import-from-derivation) to `false`.
Without IFD it is ensured that evaluation is complete and Hoffman can produce a build plan before starting any realisation.

## Example

In the following Hoffman expression, the inner derivation `drv` produces a file with contents `hello`.

```hoffman
# IFD.hoffman
let
  drv = derivation {
    name = "hello";
    builder = "/bin/sh";
    args = [ "-c" "echo -n hello > $out" ];
    system = builtins.currentSystem;
  };
in "${builtins.readFile drv} world"
```

```shellSession
hoffman-instantiate IFD.hoffman --eval --read-write-mode
```

```
building '/hoffman/store/348q1cal6sdgfxs8zqi9v8llrsn4kqkq-hello.drv'...
"hello world"
```

The contents of the derivation's output have to be [realised] before they can be read with [`readFile`](./builtins.md#builtins-readFile).
Only then evaluation can continue to produce the final result.

## Illustration

As a first approximation, the following data flow graph shows how evaluation and building are interleaved, if the value of a Hoffman expression depends on realising a [store object].
Boxes are data structures, arrow labels are transformations.

```
+----------------------+             +------------------------+
| Hoffman evaluator        |             | Hoffman store              |
|  .----------------.  |             |                        |
|  | Hoffman expression |  |             |                        |
|  '----------------'  |             |                        |
|          |           |             |                        |
|       evaluate       |             |                        |
|          |           |             |                        |
|          V           |             |                        |
|    .------------.    |             |                        |
|    | derivation |    |             |  .------------------.  |
|    | expression |----|-instantiate-|->| store derivation |  |
|    '------------'    |             |  '------------------'  |
|                      |             |           |            |
|                      |             |        realise         |
|                      |             |           |            |
|                      |             |           V            |
|  .----------------.  |             |    .--------------.    |
|  | Hoffman expression |<-|----read-----|----| store object |    |
|  '----------------'  |             |    '--------------'    |
|          |           |             |                        |
|       evaluate       |             |                        |
|          |           |             |                        |
|          V           |             |                        |
|    .------------.    |             |                        |
|    |   value    |    |             |                        |
|    '------------'    |             |                        |
+----------------------+             +------------------------+
```

In more detail, the following sequence diagram shows how the expression is evaluated step by step, and where evaluation is blocked to wait for the build output to appear.

```
.-------.     .-------------.                        .---------.
|Hoffman CLI|     |Hoffman evaluator|                        |Hoffman store|
'-------'     '-------------'                        '---------'
    |                |                                    |
    |evaluate IFD.hoffman|                                    |
    |--------------->|                                    |
    |                |                                    |
    |  evaluate `"${readFile drv} world"`                 |
    |                |                                    |
    |    evaluate `readFile drv`                          |
    |                |                                    |
    |   evaluate `drv` as string                          |
    |                |                                    |
    |                |instantiate /hoffman/store/...-hello.drv|
    |                |----------------------------------->|
    |                :                                    |
    |                :  realise /hoffman/store/...-hello.drv  |
    |                :----------------------------------->|
    |                :                                    |
    |                                                     |--------.
    |                :                                    |        |
    |      (evaluation blocked)                           |  echo hello > $out
    |                :                                    |        |
    |                                                     |<-------'
    |                :        /hoffman/store/...-hello        |
    |                |<-----------------------------------|
    |                |                                    |
    |  resume `readFile /hoffman/store/...-hello`             |
    |                |                                    |
    |                |   readFile /hoffman/store/...-hello    |
    |                |----------------------------------->|
    |                |                                    |
    |                |               hello                |
    |                |<-----------------------------------|
    |                |                                    |
    |      resume `"${"hello"} world"`                    |
    |                |                                    |
    |        resume `"hello world"`                       |
    |                |                                    |
    | "hello world"  |                                    |
    |<---------------|                                    |
.-------.     .-------------.                        .---------.
|Hoffman CLI|     |Hoffman evaluator|                        |Hoffman store|
'-------'     '-------------'                        '---------'
```
