project(
  'hoffman-expr-c',
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

deps_private_maybe_subproject = [
  dependency('hoffman-util'),
  dependency('hoffman-store'),
  dependency('hoffman-expr'),
]
deps_public_maybe_subproject = [
  dependency('hoffman-util-c'),
  dependency('hoffman-store-c'),
]
subdir('hoffman-meson-build-support/subprojects')

subdir('hoffman-meson-build-support/common')

sources = files(
  'hoffman_api_expr.cc',
  'hoffman_api_external.cc',
  'hoffman_api_value.cc',
)

include_dirs = [ include_directories('.') ]

headers = files(
  'hoffman_api_expr.h',
  'hoffman_api_expr_internal.h',
  'hoffman_api_external.h',
  'hoffman_api_value.h',
)

subdir('hoffman-meson-build-support/export-all-symbols')
subdir('hoffman-meson-build-support/windows-version')

this_library = library(
  'hoffmanexprc',
  sources,
  soversion : hoffman_soversion,
  dependencies : deps_public + deps_private + deps_other,
  include_directories : include_dirs,
  link_args : linker_export_flags,
  prelink : true, # For C++ static initializers
  install : true,
)

install_headers(headers, preserve_path : true)

libraries_private = []

subdir('hoffman-meson-build-support/export')
