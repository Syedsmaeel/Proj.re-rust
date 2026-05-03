cxx = meson.get_compiler('cpp')

deps_other = []
subdir('hoffman-meson-build-support/common/asan-options')

libstoreconsumer_tester = executable(
  'test-libstoreconsumer',
  'main.cc',
  dependencies : deps_other + [
    dependency('hoffman-store'),
  ],
  build_by_default : false,
)
