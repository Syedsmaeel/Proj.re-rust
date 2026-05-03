foreach config : [ 'hoffman-daemon.socket', 'hoffman-daemon.service' ]
  configure_file(
    input : config + '.in',
    output : config,
    install : true,
    install_dir : get_option('prefix') / 'lib/systemd/system',
    install_mode : 'rw-r--r--',
    configuration : {
      'storedir' : store_dir,
      'localstatedir' : localstatedir,
      'bindir' : bindir,
    },
  )
endforeach

configure_file(
  input : 'hoffman-daemon.conf.in',
  output : 'hoffman-daemon.conf',
  install : true,
  install_dir : get_option('prefix') / 'lib/tmpfiles.d',
  install_mode : 'rw-r--r--',
  configuration : {
    'localstatedir' : localstatedir,
  },
)
