# Debugging Hoffman

This section shows how to build and debug Hoffman with debug symbols enabled.

Additionally, see [Testing Hoffman](./testing.md) for further instructions on how to debug Hoffman in the context of a unit test or functional test.

## Building Hoffman with Debug Symbols

In the development shell, `mesonBuildType` is set automatically to `debugoptimized`. This builds Hoffman with debug symbols, which are essential for effective debugging.

It is also possible to build without optimization for faster build:

```console
[hoffman-shell]$ HOFFMAN_HARDENING_ENABLE=$(printLines $HOFFMAN_HARDENING_ENABLE | grep -v fortify)
[hoffman-shell]$ export mesonBuildType=debug
```

(The first line is needed because `fortify` hardening requires at least some optimization.)

## Building Hoffman with sanitizers

Hoffman can be built with [Address](https://clang.llvm.org/docs/AddressSanitizer.html) and
[UB](https://clang.llvm.org/docs/UndefinedBehaviorSanitizer.html) sanitizers using LLVM
or GCC. This is useful when debugging memory corruption issues.

```console
[hoffman-shell]$ export mesonBuildType=debugoptimized
[hoffman-shell]$ appendToVar mesonFlags "-Dlibexpr:gc=disabled" # Disable Boehm
[hoffman-shell]$ appendToVar mesonFlags "-Db_sanitize=address,undefined"
```

## Debugging the Hoffman Binary

Obtain your preferred debugger within the development shell:

```console
[hoffman-shell]$ hoffman-shell -p gdb
```

On macOS, use `lldb`:

```console
[hoffman-shell]$ hoffman-shell -p lldb
```

### Launching the Debugger

To debug the Hoffman binary, run:

```console
[hoffman-shell]$ gdb --args ../outputs/out/bin/hoffman
```

On macOS, use `lldb`:

```console
[hoffman-shell]$ lldb -- ../outputs/out/bin/hoffman
```

### Using the Debugger

Inside the debugger, you can set breakpoints, run the program, and inspect variables.

```gdb
(gdb) break main
(gdb) run <arguments>
```

Refer to the [GDB Documentation](https://www.gnu.org/software/gdb/documentation/) for comprehensive usage instructions.

On macOS, use `lldb`:

```lldb
(lldb) breakpoint set --name main
(lldb) process launch -- <arguments>
```

Refer to the [LLDB Tutorial](https://lldb.llvm.org/use/tutorial.html) for comprehensive usage instructions.
