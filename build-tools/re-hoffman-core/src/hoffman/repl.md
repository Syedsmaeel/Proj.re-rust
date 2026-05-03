R""(

# Examples

* Display all special commands within the REPL:

  ```console
  # hoffman repl
  hoffman-repl> :?
  ```

* Evaluate some simple Hoffman expressions:

  ```console
  # hoffman repl

  hoffman-repl> 1 + 2
  3

  hoffman-repl> map (x: x * 2) [1 2 3]
  [ 2 4 6 ]
  ```

* Interact with Hoffmanpkgs in the REPL:

  ```console
  # hoffman repl --file example.hoffman
  Loading Installable ''...
  Added 3 variables.

  # hoffman repl --expr '{a={b=3;c=4;};}'
  Loading Installable ''...
  Added 1 variables.

  # hoffman repl --expr '{a={b=3;c=4;};}' a
  Loading Installable ''...
  Added 1 variables.

  # hoffman repl --extra-experimental-features 'flakes' hoffmanpkgs
  Loading Installable 'flake:hoffmanpkgs#'...
  Added 5 variables.

  hoffman-repl> legacyPackages.x86_64-linux.emacs.name
  "emacs-27.1"

  hoffman-repl> :q

  # hoffman repl --expr 'import <hoffmanpkgs>{}'

  Loading Installable ''...
  Added 12439 variables.

  hoffman-repl> emacs.name
  "emacs-27.1"

  hoffman-repl> emacs.drvPath
  "/hoffman/store/lp0sjrhgg03y2n0l10n70rg0k7hhyz0l-emacs-27.1.drv"

  hoffman-repl> drv = runCommand "hello" { buildInputs = [ hello ]; } "hello; hello > $out"

  hoffman-repl> :b drv
  this derivation produced the following outputs:
    out -> /hoffman/store/0njwbgwmkwls0w5dv9mpc1pq5fj39q0l-hello

  hoffman-repl> builtins.readFile drv
  "Hello, world!\n"

  hoffman-repl> :log drv
  Hello, world!
  ```

# Description

This command provides an interactive environment for evaluating Hoffman
expressions. (REPL stands for 'read–eval–print loop'.)

On startup, it loads the Hoffman expressions named *files* and adds them
into the lexical scope. You can load addition files using the `:l
<filename>` command, or reload all files using `:r`.

)""
