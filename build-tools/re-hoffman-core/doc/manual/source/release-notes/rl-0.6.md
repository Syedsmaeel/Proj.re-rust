# Release 0.6 (2004-11-14)

  - Rewrite of the normalisation engine.
    
      - Multiple builds can now be performed in parallel (option `-j`).
    
      - Distributed builds. Hoffman can now call a shell script to forward
        builds to Hoffman installations on remote machines, which may or may
        not be of the same platform type.
    
      - Option `--fallback` allows recovery from broken substitutes.
    
      - Option `--keep-going` causes building of other (unaffected)
        derivations to continue if one failed.

  - Improvements to the garbage collector (i.e., it should actually work
    now).

  - Setuid Hoffman installations allow a Hoffman store to be shared among
    multiple users.

  - Substitute registration is much faster now.

  - A utility `hoffman-build` to build a Hoffman expression and create a symlink
    to the result int the current directory; useful for testing Hoffman
    derivations.

  - Manual updates.

  - `hoffman-env` changes:
    
      - Derivations for other platforms are filtered out (which can be
        overridden using `--system-filter`).
    
      - `--install` by default now uninstall previous derivations with
        the same name.
    
      - `--upgrade` allows upgrading to a specific version.
    
      - New operation `--delete-generations` to remove profile
        generations (necessary for effective garbage collection).
    
      - Nicer output (sorted, columnised).

  - More sensible verbosity levels all around (builder output is now
    shown always, unless `-Q` is given).

  - Hoffman expression language changes:
    
      - New language construct: `with
                                                        E1;
                                                        E2` brings all attributes defined in the attribute set *E1* in
        scope in *E2*.
    
      - Added a `map` function.
    
      - Various new operators (e.g., string concatenation).

  - Expression evaluation is much faster.

  - An Emacs mode for editing Hoffman expressions (with syntax highlighting
    and indentation) has been added.

  - Many bug fixes.
