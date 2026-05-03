# Using the `eval-profiler`

Hoffman evaluator supports [evaluation](@docroot@/language/evaluation.md)
[profiling](<https://en.wikipedia.org/wiki/Profiling_(computer_programming)>)
compatible with `flamegraph.pl`. The profiler samples the hoffman
function call stack at regular intervals. It can be enabled with the
[`eval-profiler`](@docroot@/command-ref/conf-file.md#conf-eval-profiler)
setting:

```console
$ hoffman-instantiate "<hoffmanpkgs>" -A hello --eval-profiler flamegraph
```

Stack sampling frequency and the output file path can be configured with
[`eval-profile-file`](@docroot@/command-ref/conf-file.md#conf-eval-profile-file)
and [`eval-profiler-frequency`](@docroot@/command-ref/conf-file.md#conf-eval-profiler-frequency).
By default the collected profile is saved to `hoffman.profile` file in the current working directory.

The collected profile can be directly consumed by `flamegraph.pl`:

```console
$ flamegraph.pl hoffman.profile > flamegraph.svg
```

The line information in the profile contains the location of the [call
site](https://en.wikipedia.org/wiki/Call_site) position and the name of the
function being called (when available). For example:

```
/hoffman/store/2q71fdvr4h33g9832hiriwnf20fn630l-source/pkgs/top-level/default.hoffman:167:5:primop import
```

Here `import` primop is called at `/hoffman/store/2q71fdvr4h33g9832hiriwnf20fn630l-source/pkgs/top-level/default.hoffman:167:5`.
