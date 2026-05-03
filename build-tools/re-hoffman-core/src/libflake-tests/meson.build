project(
  'hoffman-flake-tests',
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
  dependency('hoffman-expr-test-support'),
  dependency('hoffman-flake'),
  dependency('hoffman-flake-c'),
]
deps_public_maybe_subproject = []
subdir('hoffman-meson-build-support/subprojects')

subdir('hoffman-meson-build-support/export-all-symbols')
subdir('hoffman-meson-build-support/windows-version')

rapidcheck = dependency('rapidcheck')
deps_private += rapidcheck

gtest = dependency('gtest', main : true)
deps_private += gtest

subdir('hoffman-meson-build-support/common')

sources = files(
  'flakeref.cc',
  'hoffman_api_flake.cc',
  'url-name.cc',
)

include_dirs = [ include_directories('.') ]


this_exe = executable(
  meson.project_name(),
  sources,
  dependencies : deps_private_subproject + deps_private + deps_other,
  include_directories : include_dirs,
  # TODO: -lrapidcheck, see ../libutil-support/build.meson
  link_args : linker_export_flags + [ '-lrapidcheck' ],
  # get main from gtest
  install : true,
)

test(
  meson.project_name(),
  this_exe,
  env : {
    '_HOFFMAN_TEST_UNIT_DATA' : meson.current_source_dir() / 'data',
    'HOFFMAN_CONFIG' : 'extra-experimental-features = flakes',
    'HOME' : meson.current_build_dir() / 'test-home',
    'HOFFMAN_STORE' : '',
  },
  protocol : 'gtest',
)
