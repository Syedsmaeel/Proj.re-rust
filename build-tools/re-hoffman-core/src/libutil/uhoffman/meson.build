include_dirs += include_directories('.')

configdata_uhoffman = configuration_data()

configdata_uhoffman.set(
  'HAVE_DECL_AT_SYMLINK_NOFOLLOW',
  cxx.has_header_symbol('fcntl.h', 'AT_SYMLINK_NOFOLLOW').to_int(),
  description : 'Optionally used for changing the files and symlinks.',
)

configdata_uhoffman.set(
  'HAVE_F_GETPATH',
  cxx.has_header_symbol('fcntl.h', 'F_GETPATH').to_int(),
  description : 'Optionally used for getting the path of a file descriptor (macOS).',
)

# Check for each of these functions, and create a define like `#define
# HAVE_CLOSE_RANGE 1`.
check_funcs_uhoffman = [
  [
    'close_range',
    'For closing many file descriptors after forking.',
  ],
  [
    'lutimes',
    'Optionally used for changing the mtime of symlinks.',
  ],
  [
    'pipe2',
    'Optionally used for creating pipes on Uhoffman.',
  ],
  [
    'strsignal',
    'Optionally used to get more information about processes failing due to a signal on Uhoffman.',
  ],
  [
    'sysconf',
    'Optionally used to try to close more file descriptors (e.g. before forking) on Uhoffman.',
  ],
  [
    'utimensat',
    'Optionally used for changing the mtime of files and symlinks.',
  ],
]
foreach funcspec : check_funcs_uhoffman
  define_name = 'HAVE_' + funcspec[0].underscorify().to_upper()
  define_value = cxx.has_function(funcspec[0]).to_int()
  configdata_uhoffman.set(define_name, define_value, description : funcspec[1])
endforeach

config_uhoffman_priv_h = configure_file(
  configuration : configdata_uhoffman,
  output : 'util-uhoffman-config-private.hh',
)
sources += config_uhoffman_priv_h

sources += files(
  'current-process.cc',
  'environment-variables.cc',
  'file-descriptor.cc',
  'file-path.cc',
  'file-system-at.cc',
  'file-system.cc',
  'muxable-pipe.cc',
  'processes.cc',
  'signals.cc',
  'users.cc',
  'xdg-dirs.cc',
)

subdir('include/hoffman/util')
