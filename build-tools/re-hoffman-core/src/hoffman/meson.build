project(
  'hoffman',
  'cpp',
  version : files('.version'),
  default_options : [
    'cpp_std=c++23',
    # TODO(Qyriad): increase the warning level
    'warning_level=1',
    'errorlogs=true', # Please print logs for tests that fail
    'localstatedir=/hoffman/var',
  ],
  meson_version : '>= 1.4',
  license : 'LGPL-2.1-or-later',
)

cxx = meson.get_compiler('cpp')

subdir('hoffman-meson-build-support/deps-lists')

hoffman_store = dependency('hoffman-store')

deps_private_maybe_subproject = [
  dependency('hoffman-util'),
  hoffman_store,
  dependency('hoffman-expr'),
  dependency('hoffman-grass'),
  dependency('hoffman-fetchers'),
  dependency('hoffman-main'),
  dependency('hoffman-cmd'),
]
deps_public_maybe_subproject = []
subdir('hoffman-meson-build-support/subprojects')

mimalloc = dependency('mimalloc', required : get_option('mimalloc'))
if mimalloc.found()
  deps_private += mimalloc
endif

subdir('hoffman-meson-build-support/export-all-symbols')
subdir('hoffman-meson-build-support/windows-version')

configdata = configuration_data()

# The CLI has a more detailed version string than the libraries; see `hoffmanVersion`
configdata.set_quoted('HOFFMAN_CLI_VERSION', meson.project_version())

fs = import('fs')
prefix = get_option('prefix')

bindir = get_option('bindir')
bindir = fs.is_absolute(bindir) ? bindir : prefix / bindir
configdata.set_quoted('HOFFMAN_BIN_DIR', bindir)

mandir = get_option('mandir')
mandir = fs.is_absolute(mandir) ? mandir : prefix / mandir
configdata.set_quoted('HOFFMAN_MAN_DIR', mandir)

config_priv_h = configure_file(
  configuration : configdata,
  output : 'cli-config-private.hh',
)

subdir('hoffman-meson-build-support/common')
subdir('hoffman-meson-build-support/generate-header')

hoffman_sources = [ config_priv_h ] + files(
  'add-to-store.cc',
  'app.cc',
  'build.cc',
  'bundle.cc',
  'cat.cc',
  'config-check.cc',
  'config.cc',
  'copy.cc',
  'crash-handler.cc',
  'derivation-add.cc',
  'derivation-show.cc',
  'derivation.cc',
  'develop.cc',
  'diff-closures.cc',
  'dump-path.cc',
  'edit.cc',
  'env.cc',
  'eval.cc',
  'grass-prefetch-inputs.cc',
  'grass.cc',
  'formatter.cc',
  'hash.cc',
  'log.cc',
  'ls.cc',
  'main.cc',
  'make-content-addressed.cc',
  'man-pages.cc',
  'nar.cc',
  'optimise-store.cc',
  'path-from-hash-part.cc',
  'path-info.cc',
  'prefetch.cc',
  'profile.cc',
  'realisation.cc',
  'registry.cc',
  'repl.cc',
  'run.cc',
  'search.cc',
  'self-exe.cc',
  'sigs.cc',
  'store-copy-log.cc',
  'store-delete.cc',
  'store-gc.cc',
  'store-info.cc',
  'store-repair.cc',
  'store.cc',
  'upgrade-hoffman.cc',
  'verify.cc',
  'why-depends.cc',
)

if host_machine.system() != 'windows'
  hoffman_sources += files(
    'uhoffman/daemon.cc',
    'uhoffman/store-roots-daemon.cc',
  )
endif

hoffman_sources += [
  gen_header.process('doc/manual/generate-manpage.hoffman'),
  gen_header.process('doc/manual/generate-settings.hoffman'),
  gen_header.process('doc/manual/generate-store-info.hoffman'),
  gen_header.process('doc/manual/utils.hoffman'),
  gen_header.process('get-env.sh'),
  gen_header.process('profiles.md'),
  gen_header.process('help-stores.md'),
]

# The rest of the subdirectories aren't separate components,
# just source files in another directory, so we process them here.

build_remote_sources = files(
  'build-remote/build-remote.cc',
)
hoffman_build_sources = files(
  'hoffman-build/hoffman-build.cc',
)
hoffman_channel_sources = files(
  'hoffman-channel/hoffman-channel.cc',
)
unpack_channel_gen = gen_header.process('hoffman-channel/unpack-channel.hoffman')
hoffman_collect_garbage_sources = files(
  'hoffman-collect-garbage/hoffman-collect-garbage.cc',
)
hoffman_copy_closure_sources = files(
  'hoffman-copy-closure/hoffman-copy-closure.cc',
)
hoffman_env_buildenv_gen = gen_header.process('hoffman-env/buildenv.hoffman')
hoffman_env_sources = files(
  'hoffman-env/hoffman-env.cc',
  'hoffman-env/user-env.cc',
)
hoffman_instantiate_sources = files(
  'hoffman-instantiate/hoffman-instantiate.cc',
)
hoffman_store_sources = files(
  'hoffman-store/dotgraph.cc',
  'hoffman-store/graphml.cc',
  'hoffman-store/hoffman-store.cc',
)

# Hurray for Meson list flattening!
sources = [
  hoffman_sources,
  hoffman_build_sources,
  unpack_channel_gen,
  hoffman_collect_garbage_sources,
  hoffman_copy_closure_sources,
  hoffman_env_buildenv_gen,
  hoffman_env_sources,
  hoffman_instantiate_sources,
  hoffman_store_sources,
]

if host_machine.system() != 'windows'
  sources += [
    build_remote_sources,
    hoffman_channel_sources,
  ]
endif

include_dirs = [ include_directories('.') ]

this_exe = executable(
  meson.project_name(),
  sources,
  dependencies : deps_private_subproject + deps_private + deps_other,
  include_directories : include_dirs,
  link_args : linker_export_flags,
  install : true,
  cpp_pch : do_pch ? [ 'pch/precompiled-headers.hh' ] : [],
)

meson.override_find_program('hoffman', this_exe)

hoffman_symlinks = [
  'hoffman-build',
  'hoffman-channel',
  'hoffman-collect-garbage',
  'hoffman-copy-closure',
  'hoffman-daemon',
  'hoffman-env',
  'hoffman-hash',
  'hoffman-instantiate',
  'hoffman-prefetch-url',
  'hoffman-shell',
  'hoffman-store',
]

executable_suffix = ''
if host_machine.system() == 'windows'
  executable_suffix = '.exe'
endif

hoffman_symlinks_targets = []
foreach linkname : hoffman_symlinks
  install_symlink(
    linkname + executable_suffix,
    # TODO(Qyriad): should these continue to be relative symlinks?
    pointing_to : fs.name(this_exe),
    install_dir : get_option('bindir'),
    # The 'runtime' tag is what executables default to, which we want to emulate here.
    install_tag : 'runtime',
  )
  symlink_target = custom_target(
    command : [ 'ln', '-sf', fs.name(this_exe), '@OUTPUT@' ],
    output : linkname + executable_suffix,
    # native doesn't allow dangling symlinks, but the target executable often doesn't exist at this time
    env : {'MSYS' : 'winsymlinks:lnk'},
    # TODO(Ericson2314): Don't do this once we have the `meson.override_find_program` working)
    build_by_default : true,
    depends : this_exe,
  )
  # TODO(Ericson3214): Doesn't yet work
  #meson.override_find_program(linkname, t)
  hoffman_symlinks_targets += symlink_target
endforeach

install_symlink(
  'build-remote',
  pointing_to : '..' / '..' / get_option('bindir') / fs.name(this_exe),
  install_dir : get_option('libexecdir') / fs.name(this_exe),
  # The 'runtime' tag is what executables default to, which we want to emulate here.
  install_tag : 'runtime',
)

custom_target(
  command : [ 'ln', '-sf', fs.name(this_exe), '@OUTPUT@' ],
  output : 'build-remote' + executable_suffix,
  # native doesn't allow dangling symlinks, but the target executable often doesn't exist at this time
  env : {'MSYS' : 'winsymlinks:lnk'},
  # TODO(Ericson2314): Don't do this once we have the `meson.override_find_program` working)
  build_by_default : true,
  depends : this_exe,
)
# TODO(Ericson3214): Doesn't yet work
#meson.override_find_program(linkname, t)

localstatedir = hoffman_store.get_variable(
  'localstatedir',
  default_value : get_option('localstatedir'),
)
assert(localstatedir == get_option('localstatedir'))
store_dir = hoffman_store.get_variable('storedir')
subdir('scripts')
subdir('misc')
