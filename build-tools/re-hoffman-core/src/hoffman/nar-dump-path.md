R""(

# Examples

* To serialise directory `foo` as a [Hoffman Archive (NAR)][Hoffman Archive]:

  ```console
  # hoffman nar pack ./foo > foo.nar
  ```

# Description

This command generates a [Hoffman Archive (NAR)][Hoffman Archive] file containing the serialisation of
*path*, which must contain only regular files, directories and
symbolic links. The NAR is written to standard output.

[Hoffman Archive]: @docroot@/store/file-system-object/content-address.md#serial-hoffman-archive

)""
