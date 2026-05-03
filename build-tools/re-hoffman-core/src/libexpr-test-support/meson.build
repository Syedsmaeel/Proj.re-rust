project(
  'hoffman-expr-test-support',
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
  dependency('hoffman-util-test-support'),
  dependency('hoffman-store'),
  dependency('hoffman-store-test-support'),
  dependency('hoffman-expr'),
  dependency('hoffman-expr-c'),
]
subdir('hoffman-meson-build-support/subprojects')

rapidcheck = dependency('rapidcheck')
deps_public += rapidcheck

subdir('hoffman-meson-build-support/common')

sources = files(
  'tests/value/context.cc',
)

subdir('include/hoffman/expr/tests')

subdir('hoffman-meson-build-support/export-all-symbols')
subdir('hoffman-meson-build-support/windows-version')

this_library = library(
  'hoffman-expr-test-support',
  sources,
  soversion : hoffman_soversion,
  dependencies : deps_public + deps_private + deps_other,
  include_directories : include_dirs,
  # TODO: Remove `-lrapidcheck` when https://github.com/emil-e/rapidcheck/pull/326
  #       is available. See also ../libutil/build.meson
  link_args : linker_export_flags + [ '-lrapidcheck' ],
  prelink : true, # For C++ static initializers
  install : true,
)

install_headers(headers, subdir : 'hoffman/expr/tests', preserve_path : true)

libraries_private = []

subdir('hoffman-meson-build-support/export')
