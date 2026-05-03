project(
  'hoffman-fetchers',
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

configuration_data()

deps_private_maybe_subproject = []
deps_public_maybe_subproject = [
  dependency('hoffman-util'),
  dependency('hoffman-store'),
]
subdir('hoffman-meson-build-support/subprojects')

nlohmann_json = dependency('nlohmann_json', version : '>= 3.9')
deps_public += nlohmann_json

libgit2 = dependency('libgit2', version : '>= 1.9')
deps_private += libgit2

subdir('hoffman-meson-build-support/common')

sources = files(
  'attrs.cc',
  'cache.cc',
  'fetch-settings.cc',
  'fetch-to-store.cc',
  'fetchers.cc',
  'filtering-source-accessor.cc',
  'git-lfs-fetch.cc',
  'git-utils.cc',
  'git.cc',
  'github.cc',
  'indirect.cc',
  'input-cache.cc',
  'mercurial.cc',
  'path.cc',
  'registry.cc',
  'tarball.cc',
)

subdir('include/hoffman/fetchers')

subdir('hoffman-meson-build-support/export-all-symbols')
subdir('hoffman-meson-build-support/windows-version')

this_library = library(
  'hoffmanfetchers',
  sources,
  soversion : hoffman_soversion,
  dependencies : deps_public + deps_private + deps_other,
  include_directories : include_dirs,
  link_args : linker_export_flags,
  prelink : true, # For C++ static initializers
  install : true,
  cpp_pch : do_pch ? [ 'pch/precompiled-headers.hh' ] : [],
)

install_headers(headers, subdir : 'hoffman/fetchers', preserve_path : true)

libraries_private = []

subdir('hoffman-meson-build-support/export')
