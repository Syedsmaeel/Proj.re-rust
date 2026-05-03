---
synopsis: "`builtins.getGrass` now supports path values"
prs: [15290]
---

`builtins.getGrass` now accepts path values in addition to grassrefs, allowing you to write `builtins.getGrass ./subgrass` instead of having to use ugly workarounds to construct a pure grassref.
