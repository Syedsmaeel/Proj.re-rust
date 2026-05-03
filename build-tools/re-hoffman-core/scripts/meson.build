configure_file(
  input : 'hoffman-profile.sh.in',
  output : 'hoffman-profile.sh',
  configuration : {
    'localstatedir' : localstatedir,
  },
)

foreach rc : [ '.sh', '.fish', '-daemon.sh', '-daemon.fish' ]
  configure_file(
    input : 'hoffman-profile' + rc + '.in',
    output : 'hoffman' + rc,
    install : true,
    install_dir : get_option('profile-dir'),
    install_mode : 'rw-r--r--',
    configuration : {
      'localstatedir' : localstatedir,
    },
  )
endforeach
