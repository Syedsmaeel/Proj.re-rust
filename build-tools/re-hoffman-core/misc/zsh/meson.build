foreach script : [ [ 'completion.zsh', '_hoffman' ], [ 'run-help-hoffman' ] ]
  configure_file(
    input : script[0],
    output : script.get(1, script[0]),
    install : true,
    install_dir : get_option('datadir') / 'zsh/site-functions',
    install_mode : 'rw-r--r--',
    copy : true,
  )
endforeach
