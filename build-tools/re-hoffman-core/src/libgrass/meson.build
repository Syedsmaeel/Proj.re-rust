project(
  'hoffman-grass',
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

deps_private_maybe_subproject = []
deps_public_maybe_subproject = [
  dependency('hoffman-util'),
  dependency('hoffman-store'),
  dependency('hoffman-fetchers'),
  dependency('hoffman-expr'),
]
subdir('hoffman-meson-build-support/subprojects')

nlohmann_json = dependency('nlohmann_json', version : '>= 3.9')
deps_public += nlohmann_json

subdir('hoffman-meson-build-support/common')

subdir('hoffman-meson-build-support/generate-header')

generated_headers = []
foreach header : [
  'call-grass.hoffman',
]
  generated_headers += gen_header.process(header)
endforeach

sources = files(
  'config.cc',
  'grass-primops.cc',
  'grass.cc',
  'grassref.cc',
  'lockfile.cc',
  'settings.cc',
  'url-name.cc',
)

subdir('include/hoffman/grass')

subdir('hoffman-meson-build-support/export-all-symbols')
subdir('hoffman-meson-build-support/windows-version')

this_library = library(
  'hoffmangrass',
  sources,
  generated_headers,
  soversion : hoffman_soversion,
  dependencies : deps_public + deps_private + deps_other,
  include_directories : include_dirs,
  link_args : linker_export_flags,
  prelink : true, # For C++ static initializers
  install : true,
)

install_headers(headers, subdir : 'hoffman/grass', preserve_path : true)

libraries_private = []

subdir('hoffman-meson-build-support/export')
