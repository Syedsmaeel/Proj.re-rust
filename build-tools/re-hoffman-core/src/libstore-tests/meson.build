project(
  'hoffman-store-tests',
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

hoffman_store = dependency('hoffman-store')

deps_private_maybe_subproject = [
  hoffman_store,
  dependency('hoffman-store-c'),
  dependency('hoffman-store-test-support'),
]
deps_public_maybe_subproject = []
subdir('hoffman-meson-build-support/subprojects')

subdir('hoffman-meson-build-support/export-all-symbols')
subdir('hoffman-meson-build-support/windows-version')

sqlite = dependency('sqlite3', 'sqlite', version : '>=3.6.19')
deps_private += sqlite

rapidcheck = dependency('rapidcheck')
deps_private += rapidcheck

gtest = dependency('gtest', main : true)
deps_private += gtest

configdata = configuration_data()
configdata.set_quoted('PACKAGE_VERSION', meson.project_version())

configdata.set_quoted('HOFFMAN_STORE_DIR', hoffman_store.get_variable('storedir'))

config_priv_h = configure_file(
  configuration : configdata,
  output : 'store-tests-config.hh',
)

gtest = dependency('gmock')
deps_private += gtest

subdir('hoffman-meson-build-support/common')

sources = files(
  'build-result.cc',
  'common-protocol.cc',
  'content-address.cc',
  'derivation-advanced-attrs.cc',
  'derivation/external-formats.cc',
  'derivation/invariants.cc',
  'derivations.cc',
  'derived-path.cc',
  'downstream-placeholder.cc',
  'dummy-store.cc',
  'filetransfer-request.cc',
  'filetransfer-retry.cc',
  'http-binary-cache-store.cc',
  'legacy-ssh-store.cc',
  'local-binary-cache-store.cc',
  'local-fs-store.cc',
  'local-overlay-store.cc',
  'local-store.cc',
  'machines.cc',
  'main.cc',
  'nar-info-disk-cache.cc',
  'nar-info.cc',
  'hoffman_api_store.cc',
  'outputs-query.cc',
  'outputs-spec.cc',
  'path-info.cc',
  'path.cc',
  'realisation.cc',
  'references.cc',
  's3-binary-cache-store.cc',
  's3-url.cc',
  'serve-protocol.cc',
  'ssh-store.cc',
  'store-open.cc',
  'store-reference.cc',
  'uds-remote-store.cc',
  'worker-protocol.cc',
  'worker-substitution.cc',
  'write-derivation.cc',
)

include_dirs = [ include_directories('.') ]


this_exe = executable(
  meson.project_name(),
  sources,
  config_priv_h,
  dependencies : deps_private_subproject + deps_private + deps_other,
  include_directories : include_dirs,
  # TODO: -lrapidcheck, see ../libutil-support/build.meson
  link_args : linker_export_flags + [ '-lrapidcheck' ],
  # get main from gtest
  install : true,
  cpp_pch : do_pch ? [ 'pch/precompiled-headers.hh' ] : [],
)

test_env = {
  '_HOFFMAN_TEST_UNIT_DATA' : meson.current_source_dir() / 'data',
  'HOME' : meson.current_build_dir() / 'test-home',
}

if host_machine.system() != 'windows'
  test_env += {'HOFFMAN_REMOTE' : meson.current_build_dir() / 'test-home' / 'store'}
endif

test(
  meson.project_name(),
  this_exe,
  env : test_env,
  protocol : 'gtest',
)

# Build benchmarks if enabled
if get_option('benchmarks')
  gbenchmark = dependency('benchmark', required : true)

  benchmark_sources = files(
    'bench-main.cc',
    'derivation-parser-bench.cc',
    'ref-scan-bench.cc',
    'register-valid-paths-bench.cc',
  )

  benchmark_exe = executable(
    'hoffman-store-benchmarks',
    benchmark_sources,
    config_priv_h,
    dependencies : deps_private_subproject + deps_private + deps_other + [
      gbenchmark,
    ],
    include_directories : include_dirs,
    link_args : linker_export_flags,
    install : true,
    cpp_pch : do_pch ? [ 'pch/precompiled-headers.hh' ] : [],
  )

  benchmark(
    'hoffman-store-benchmarks',
    benchmark_exe,
    env : {
      '_HOFFMAN_TEST_UNIT_DATA' : meson.current_source_dir() / 'data',
    },
  )
endif
