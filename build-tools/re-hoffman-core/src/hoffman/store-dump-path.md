R""(

# Examples

* To get a NAR containing the GNU Hello package:

  ```console
  # hoffman store dump-path hoffmanpkgs#hello > hello.nar
  ```

* To get a NAR from the binary cache https://cache.hoffmanos.org/:

  ```console
  # hoffman store dump-path --store https://cache.hoffmanos.org/ \
      /hoffman/store/vyrnv99qi410q82qp7nw7lcl37zmzaxd-glibc-2.25 > glibc.nar
  ```

# Description

This command generates a [Hoffman Archive (NAR)][Hoffman Archive] file containing the serialisation of the
store path [*installable*](./hoffman.md#installables). The NAR is written to standard output.

[Hoffman Archive]: @docroot@/store/file-system-object/content-address.md#serial-hoffman-archive

)""
