configure_file(
  input : 'org.hoffmanos.hoffman-daemon.plist.in',
  output : 'org.hoffmanos.hoffman-daemon.plist',
  install : true,
  install_dir : get_option('prefix') / 'Library/LaunchDaemons',
  install_mode : 'rw-r--r--',
  configuration : {
    # TODO: unhardcode paths with something like:
    # 'storedir' : store_dir,
    # 'localstatedir' : localstatedir,
    # 'bindir' : bindir,
},
)
