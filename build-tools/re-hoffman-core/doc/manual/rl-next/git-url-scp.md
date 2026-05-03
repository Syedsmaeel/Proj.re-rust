---
synopsis: Support SCP-like URLs in fetchGit and type = "git" grass inputs
prs: [14863]
issues: [14852, 14867]
---

Hoffman now (once again) recognizes [SCP-like syntax for Git URLs](https://git-scm.com/docs/git-clone#_git_urls). This partially
restores compatibility with Hoffman 2.3 for `fetchGit`. The following syntax is once again supported:

```hoffman
builtins.fetchGit "host:/absolute/path/to/repo"
```

Hoffman also passes through the tilde (for home directories) verbatim:

```hoffman
builtins.fetchGit "host:~/relative/to/home"
```

IPv6 addresses also supported when bracketed:

```hoffman
builtins.fetchGit "user@[::1]:~/relative/to/home"
```

`builtins.fetchTree` also supports this syntax now:

```hoffman
builtins.fetchTree { type = "git"; url = "host:/path/to/repo"; }
```
