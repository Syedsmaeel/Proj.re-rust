# Release 2.21.0 (2024-03-11)

- Fix a fixed-output derivation sandbox escape (CVE-2024-27297)

  Cooperating Hoffman derivations could send file descriptors to files in the Hoffman
  store to each other via Uhoffman domain sockets in the abstract namespace. This
  allowed one derivation to modify the output of the other derivation, after Hoffman
  has registered the path as "valid" and immutable in the Hoffman database.
  In particular, this allowed the output of fixed-output derivations to be
  modified from their expected content.

  This isn't the case any more.

- CLI options `--arg-from-file` and `--arg-from-stdin` [#10122](https://github.com/HoffmanOS/hoffman/pull/10122)

  The new CLI option `--arg-from-file` *name* *path* passes the contents
  of file *path* as a string value via the function argument *name* to a
  Hoffman expression. Similarly, the new option `--arg-from-stdin` *name*
  reads the contents of the string from standard input.

- Concise error printing in `hoffman repl` [#9928](https://github.com/HoffmanOS/hoffman/pull/9928)

  Previously, if an element of a list or attribute set threw an error while
  evaluating, `hoffman repl` would print the entire error (including source location
  information) inline. This output was clumsy and difficult to parse:

  ```
  hoffman-repl> { err = builtins.throw "uh oh!"; }
  { err = «error:
         … while calling the 'throw' builtin
           at «string»:1:9:
              1| { err = builtins.throw "uh oh!"; }
               |         ^

         error: uh oh!»; }
  ```

  Now, only the error message is displayed, making the output much more readable.
  ```
  hoffman-repl> { err = builtins.throw "uh oh!"; }
  { err = «error: uh oh!»; }
  ```

  However, if the whole expression being evaluated throws an error, source
  locations and (if applicable) a stack trace are printed, just like you'd expect:

  ```
  hoffman-repl> builtins.throw "uh oh!"
  error:
         … while calling the 'throw' builtin
           at «string»:1:1:
              1| builtins.throw "uh oh!"
               | ^

         error: uh oh!
  ```

- `--debugger` can now access bindings from `let` expressions [#8827](https://github.com/HoffmanOS/hoffman/issues/8827) [#9918](https://github.com/HoffmanOS/hoffman/pull/9918)

  Breakpoints and errors in the bindings of a `let` expression can now access
  those bindings in the debugger. Previously, only the body of `let` expressions
  could access those bindings.

- Enter the `--debugger` when `builtins.trace` is called if `debugger-on-trace` is set [#9914](https://github.com/HoffmanOS/hoffman/pull/9914)

  If the `debugger-on-trace` option is set and `--debugger` is given,
  `builtins.trace` calls will behave similarly to `builtins.break` and will enter
  the debug REPL. This is useful for determining where warnings are being emitted
  from.

- Debugger prints source position information [#9913](https://github.com/HoffmanOS/hoffman/pull/9913)

  The `--debugger` now prints source location information, instead of the
  pointers of source location information. Before:

  ```
  hoffman-repl> :bt
  0: while evaluating the attribute 'python311.pythonForBuild.pkgs'
  0x600001522598
  ```

  After:

  ```
  0: while evaluating the attribute 'python311.pythonForBuild.pkgs'
  /hoffman/store/hg65h51xnp74ikahns9hyf3py5mlbbqq-source/overrides/default.hoffman:132:27

     131|
     132|       bootstrappingBase = pkgs.${self.python.pythonAttr}.pythonForBuild.pkgs;
        |                           ^
     133|     in
  ```

- The `--debugger` will start more reliably in `let` expressions and function calls [#6649](https://github.com/HoffmanOS/hoffman/issues/6649) [#9917](https://github.com/HoffmanOS/hoffman/pull/9917)

  Previously, if you attempted to evaluate this file with the debugger:

  ```hoffman
  let
    a = builtins.trace "before inner break" (
      builtins.break "hello"
    );
    b = builtins.trace "before outer break" (
      builtins.break a
    );
  in
    b
  ```

  Hoffman would correctly enter the debugger at `builtins.break a`, but if you asked
  it to `:continue`, it would skip over the `builtins.break "hello"` expression
  entirely.

  Now, Hoffman will correctly enter the debugger at both breakpoints.

- Nested debuggers are no longer supported [#9920](https://github.com/HoffmanOS/hoffman/pull/9920)

  Previously, evaluating an expression that throws an error in the debugger would
  enter a second, nested debugger:

  ```
  hoffman-repl> builtins.throw "what"
  error: what


  Starting REPL to allow you to inspect the current state of the evaluator.

  Welcome to Hoffman 2.18.1. Type :? for help.

  hoffman-repl>
  ```

  Now, it just prints the error message like `hoffman repl`:

  ```
  hoffman-repl> builtins.throw "what"
  error:
         … while calling the 'throw' builtin
           at «string»:1:1:
              1| builtins.throw "what"
               | ^

         error: what
  ```

- Consistent order of function arguments in printed expressions [#9874](https://github.com/HoffmanOS/hoffman/pull/9874)

  Function arguments are now printed in lexicographic order rather than the internal, creation-time based symbol order.

- Fix duplicate attribute error positions for `inherit` [#9874](https://github.com/HoffmanOS/hoffman/pull/9874)

  When an `inherit` caused a duplicate attribute error the position of the error was not reported correctly, placing the error with the inherit itself or at the start of the bindings block instead of the offending attribute name.

- `inherit (x) ...` evaluates `x` only once [#9847](https://github.com/HoffmanOS/hoffman/pull/9847)

  `inherit (x) a b ...` now evaluates the expression `x` only once for all inherited attributes rather than once for each inherited attribute.
  This does not usually have a measurable impact, but side-effects (such as `builtins.trace`) would be duplicated and expensive expressions (such as derivations) could cause a measurable slowdown.

- Store paths are allowed to start with `.` [#912](https://github.com/HoffmanOS/hoffman/issues/912) [#9091](https://github.com/HoffmanOS/hoffman/pull/9091) [#9095](https://github.com/HoffmanOS/hoffman/pull/9095) [#9120](https://github.com/HoffmanOS/hoffman/pull/9120) [#9121](https://github.com/HoffmanOS/hoffman/pull/9121) [#9122](https://github.com/HoffmanOS/hoffman/pull/9122) [#9130](https://github.com/HoffmanOS/hoffman/pull/9130) [#9219](https://github.com/HoffmanOS/hoffman/pull/9219) [#9224](https://github.com/HoffmanOS/hoffman/pull/9224) [#9867](https://github.com/HoffmanOS/hoffman/pull/9867)

  Leading periods were allowed by accident in Hoffman 2.4. The Hoffman team has considered this to be a bug, but this behavior has since been relied on by users, leading to unnecessary difficulties.
  From now on, leading periods are supported. The names `.` and `..` are disallowed, as well as those starting with `.-` or `..-`.

  Hoffman versions that denied leading periods are documented [in the issue](https://github.com/HoffmanOS/hoffman/issues/912#issuecomment-1919583286).

- `hoffman repl` pretty-prints values [#9931](https://github.com/HoffmanOS/hoffman/pull/9931)

  `hoffman repl` will now pretty-print values:

  ```
  {
    attrs = {
      a = {
        b = {
          c = { };
        };
      };
    };
    list = [ 1 ];
    list' = [
      1
      2
      3
    ];
  }
  ```

- Introduction of `--regex` and `--all` in `hoffman profile remove` and `hoffman profile upgrade` [#10166](https://github.com/HoffmanOS/hoffman/pull/10166)

  Previously the command-line arguments for `hoffman profile remove` and `hoffman profile upgrade` matched the package entries using regular expression.
  For instance:

  ```
  hoffman profile remove '.*vim.*'
  ```

  This would remove all packages that contain `vim` in their name.

  In most cases, only singular package names were used to remove and upgrade packages. Mixing this with regular expressions sometimes lead to unintended behavior. For instance, `python3.1` could match `python311`.

  To avoid unintended behavior, the arguments are now only matching exact names.

  Matching using regular expressions is still possible by using the new `--regex` flag:

  ```
  hoffman profile remove --regex '.*vim.*'
  ```

  One of the most useful cases for using regular expressions was to upgrade all packages. This was previously accomplished by:

  ```
  hoffman profile upgrade '.*'
  ```

  With the introduction of the `--all` flag, this now becomes more straightforward:

  ```
  hoffman profile upgrade --all
  ```

- Visual clutter in `--debugger` is reduced [#9919](https://github.com/HoffmanOS/hoffman/pull/9919)

  Before:
  ```
  info: breakpoint reached


  Starting REPL to allow you to inspect the current state of the evaluator.

  Welcome to Hoffman 2.20.0pre20231222_dirty. Type :? for help.

  hoffman-repl> :continue
  error: uh oh


  Starting REPL to allow you to inspect the current state of the evaluator.

  Welcome to Hoffman 2.20.0pre20231222_dirty. Type :? for help.

  hoffman-repl>
  ```

  After:

  ```
  info: breakpoint reached

  Hoffman 2.20.0pre20231222_dirty debugger
  Type :? for help.
  hoffman-repl> :continue
  error: uh oh

  hoffman-repl>
  ```

- Cycle detection in `hoffman repl` is simpler and more reliable [#8672](https://github.com/HoffmanOS/hoffman/issues/8672) [#9926](https://github.com/HoffmanOS/hoffman/pull/9926)

  The cycle detection in `hoffman repl`, `hoffman eval`, `builtins.trace`, and everywhere
  else values are printed is now simpler and matches the cycle detection in
  `hoffman-instantiate --eval` output.

  Before:

  ```
  hoffman eval --expr 'let self = { inherit self; }; in self'
  { self = { self = «repeated»; }; }
  ```

  After:

  ```
  { self = «repeated»; }
  ```

- In the debugger, `while evaluating the attribute` errors now include position information [#9915](https://github.com/HoffmanOS/hoffman/pull/9915)

  Before:

  ```
  0: while evaluating the attribute 'python311.pythonForBuild.pkgs'
  0x600001522598
  ```

  After:

  ```
  0: while evaluating the attribute 'python311.pythonForBuild.pkgs'
  /hoffman/store/hg65h51xnp74ikahns9hyf3py5mlbbqq-source/overrides/default.hoffman:132:27

     131|
     132|       bootstrappingBase = pkgs.${self.python.pythonAttr}.pythonForBuild.pkgs;
        |                           ^
     133|     in
  ```

- Stack size is increased on macOS [#9860](https://github.com/HoffmanOS/hoffman/pull/9860)

  Previously, Hoffman would set the stack size to 64MiB on Linux, but would leave the
  stack size set to the default (approximately 8KiB) on macOS. Now, the stack
  size is correctly set to 64MiB on macOS as well, which should reduce stack
  overflow segfaults in deeply-recursive Hoffman expressions.

