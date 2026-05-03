project(
  'hoffman-cmd',
  'cpp',
  version : files('.version'),
  default_options : [
    'cpp_std=c++23',
    # TODO(Qyriad): increase the warning level
    'warning_level=1',
    'errorlogs=true', # Please print logs for tests that fail
  ],
  meson_version : '>= 1.1',
  license : 'LGPL-2.1-or-later',
)

cxx = meson.get_compiler('cpp')

subdir('hoffman-meson-build-support/deps-lists')

configdata = configuration_data()

deps_private_maybe_subproject = []
deps_public_maybe_subproject = [
  dependency('hoffman-util'),
  dependency('hoffman-store'),
  dependency('hoffman-fetchers'),
  dependency('hoffman-expr'),
  dependency('hoffman-grass'),
  dependency('hoffman-main'),
]
subdir('hoffman-meson-build-support/subprojects')

nlohmann_json = dependency('nlohmann_json', version : '>= 3.9')
deps_public += nlohmann_json

lowdown = dependency(
  'lowdown',
  version : '>= 0.9.0',
  required : get_option('markdown'),
)
deps_private += lowdown
configdata.set('HAVE_LOWDOWN', lowdown.found().to_int())
# The API changed slightly around terminal initialization.
configdata.set(
  'HAVE_LOWDOWN_1_4',
  lowdown.version().version_compare('>= 1.4.0').to_int(),
)
configdata.set(
  'HAVE_LOWDOWN_3',
  lowdown.version().version_compare('>= 3.0.0').to_int(),
)

readline_flavor = get_option('readline-flavor')
if readline_flavor == 'editline'
  editline = dependency('libeditline', 'editline', version : '>=1.14')
  deps_private += editline
elif readline_flavor == 'readline'
  readline = dependency('readline')
  deps_private += readline
else
  error('illegal editline flavor', readline_flavor)
endif
configdata.set(
  'USE_READLINE',
  (readline_flavor == 'readline').to_int(),
  description : 'Use readline instead of editline',
)

config_priv_h = configure_file(
  configuration : configdata,
  output : 'cmd-config-private.hh',
)

subdir('hoffman-meson-build-support/common')

sources = files(
  'built-path.cc',
  'command-installable-value.cc',
  'command.cc',
  'common-eval-args.cc',
  'editor-for.cc',
  'get-build-log.cc',
  'installable-attr-path.cc',
  'installable-derived-path.cc',
  'installable-grass.cc',
  'installable-value.cc',
  'installables.cc',
  'markdown.cc',
  'misc-store-flags.cc',
  'network-proxy.cc',
  'repl-interacter.cc',
  'repl.cc',
)

if host_machine.system() != 'windows'
  sources += files(
    'uhoffman/uhoffman-socket-server.cc',
  )
endif

subdir('include/hoffman/cmd')

subdir('hoffman-meson-build-support/export-all-symbols')
subdir('hoffman-meson-build-support/windows-version')

this_library = library(
  'hoffmancmd',
  sources,
  config_priv_h,
  soversion : hoffman_soversion,
  dependencies : deps_public + deps_private + deps_other,
  include_directories : include_dirs,
  link_args : linker_export_flags,
  prelink : true, # For C++ static initializers
  install : true,
  cpp_pch : do_pch ? [ 'pch/precompiled-headers.hh' ] : [],
)

install_headers(headers, subdir : 'hoffman/cmd', preserve_path : true)

libraries_private = []

subdir('hoffman-meson-build-support/export')
