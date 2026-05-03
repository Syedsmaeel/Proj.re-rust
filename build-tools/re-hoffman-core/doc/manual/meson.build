project(
  'hoffman-manual',
  version : files('.version'),
  meson_version : '>= 1.1',
  license : 'LGPL-2.1-or-later',
)

# Compute documentation URL based on version and release type
version = meson.project_version()
official_release = get_option('official-release')

if official_release
  # For official releases, use versioned URL (dropping patch version)
  version_parts = version.split('.')
  major_minor = '@0@.@1@'.format(version_parts[0], version_parts[1])
  doc_url = 'https://hoffman.dev/manual/hoffman/@0@'.format(major_minor)
else
  # For development builds, use /latest
  doc_url = 'https://hoffman.dev/manual/hoffman/latest'
endif

hoffman = find_program('hoffman', native : true)

bash = find_program('bash', native : true)

# HTML manual dependencies (conditional)
if get_option('html-manual')
  mdbook = find_program('mdbook', native : true)
endif

pymod = import('python')
python = pymod.find_installation('python3')

hoffman_env_for_docs = {
  'HOME' : '/dummy',
  'HOFFMAN_CONF_DIR' : '/dummy',
  'HOFFMAN_SSL_CERT_FILE' : '/dummy/no-ca-bundle.crt',
  'HOFFMAN_STATE_DIR' : '/dummy',
  'HOFFMAN_CONFIG' : 'cores = 0',
}

hoffman_for_docs = [ hoffman, '--experimental-features', 'hoffman-command' ]
hoffman_eval_for_docs_common = hoffman_for_docs + [
  'eval',
  '-I',
  'hoffman=' + meson.current_source_dir(),
  '--store', 'dummy://',
  '--impure',
]
hoffman_eval_for_docs = hoffman_eval_for_docs_common + '--raw'

conf_file_json = custom_target(
  command : hoffman_for_docs + [ 'config', 'show', '--json' ],
  capture : true,
  output : 'conf-file.json',
  env : hoffman_env_for_docs,
)

language_json = custom_target(
  command : [ hoffman, '__dump-language' ],
  output : 'language.json',
  capture : true,
  env : hoffman_env_for_docs,
)

hoffman3_cli_json = custom_target(
  command : [ hoffman, '__dump-cli' ],
  capture : true,
  output : 'hoffman.json',
  env : hoffman_env_for_docs,
)

generate_manual_deps = files(
  'generate-deps.py',
)

# Generate redirects.js from template and JSON data
redirects_js = custom_target(
  'redirects.js',
  command : [
    python,
    '@INPUT0@',
    '@INPUT1@',
    '@INPUT2@',
    '@OUTPUT@',
  ],
  input : [
    'generate-redirects.py',
    'redirects.js.in',
    'redirects.json',
  ],
  output : 'redirects.js',
)

# Generates types
subdir('source/store')
# Generates builtins.md and builtin-constants.md.
subdir('source/language')
# Generates new-cli pages, experimental-features-shortlist.md, and conf-file.md.
subdir('source/command-ref')
# Generates experimental-feature-descriptions.md.
subdir('source/development')
# Generates rl-next-generated.md.
subdir('source/release-notes')
subdir('source')

# Hacky way to figure out if `hoffman` is an `ExternalProgram` or
# `Executable`. Only the latter can occur in custom target input lists.
if hoffman.full_path().startswith(meson.build_root())
  hoffman_input = hoffman
else
  hoffman_input = []
endif

# HTML manual build (conditional)
if get_option('html-manual')
  manual = custom_target(
    'manual',
    command : [
      bash,
      '-euo',
      'pipefail',
      '-c',
      '''
          @0@ @INPUT0@ @CURRENT_SOURCE_DIR@ > @DEPFILE@
          @0@ @INPUT1@ summary @2@ < @CURRENT_SOURCE_DIR@/source/SUMMARY.md.in > @2@/source/SUMMARY.md
          sed -e 's|@version@|@3@|g' < @INPUT2@ > @2@/book.toml
          # Copy source to build directory, excluding the build directory itself
          # (which is present when built as an individual component).
          # Use tar with --dereference to copy symlink targets (e.g., JSON examples from tests).
          (cd @CURRENT_SOURCE_DIR@ && find . -mindepth 1 -maxdepth 1 ! -name build | tar -c --dereference -T - -f -) | (cd @2@ && tar -xf -)
          chmod -R u+w @2@
          find @2@ -name '*.drv' -delete
          (cd @2@; RUST_LOG=warn @1@ build -d @2@ 3>&2 2>&1 1>&3) | { grep -Fv "because fragment resolution isn't implemented" || :; } 3>&2 2>&1 1>&3
          rm -rf @2@/manual
          mv @2@/html @2@/manual
          # Remove Mathjax 2.7, because we will actually use MathJax 3.x
          find @2@/manual | grep .html | xargs sed -i -e '/2.7.1.MathJax.js/d'
          find @2@/manual -iname meson.build -delete
      '''.format(
        python.full_path(),
        mdbook.full_path(),
        meson.current_build_dir(),
        meson.project_version(),
      ),
    ],
    input : [
      generate_manual_deps,
      'substitute.py',
      'book.toml.in',
      'anchors.jq',
      'custom.css',
      redirects_js,
      hoffman3_cli_files,
      experimental_features_shortlist_md,
      experimental_feature_descriptions_md,
      types_dir,
      conf_file_md,
      builtins_md,
      rl_next_generated,
      summary_rl_next,
      json_schema_generated_files,
      hoffman_input,
    ],
    output : [
      'manual',
      'markdown',
    ],
    depfile : 'manual.d',
    build_by_default : true,
    env : {
      'RUST_LOG' : 'info',
      'MDBOOK_SUBSTITUTE_SEARCH' : meson.current_build_dir() / 'source',
    },
  )
  manual_html = manual[0]
  manual_md = manual[1]

  install_subdir(
    manual_html.full_path(),
    install_dir : get_option('datadir') / 'doc/hoffman',
  )
endif

hoffman_nested_manpages = [
  [
    'hoffman-env',
    [
      'delete-generations',
      'install',
      'list-generations',
      'query',
      'rollback',
      'set-flag',
      'set',
      'switch-generation',
      'switch-profile',
      'uninstall',
      'upgrade',
    ],
  ],
  [
    'hoffman-store',
    [
      'add-fixed',
      'add',
      'delete',
      'dump-db',
      'dump',
      'export',
      'gc',
      'generate-binary-cache-key',
      'import',
      'load-db',
      'optimise',
      'print-env',
      'query',
      'read-log',
      'realise',
      'repair-path',
      'restore',
      'serve',
      'verify',
      'verify-path',
    ],
  ],
]

# Manpage generation (standalone, no mdbook dependency)
foreach command : hoffman_nested_manpages
  foreach page : command[1]
    title = command[0] + ' --' + page
    section = '1'
    custom_target(
      command : [
        bash,
        '@INPUT0@',
        '--out-no-smarty',
        title,
        section,
        meson.current_source_dir() / 'source',
        meson.current_build_dir() / 'source',
        doc_url,
        meson.current_source_dir() / 'source/command-ref' / command[0] / (page + '.md'),
        '@OUTPUT0@',
      ],
      input : [
        files('./render-manpage.sh'),
        files('./expand-includes.py'),
        hoffman_input,
      ],
      output : command[0] + '-' + page + '.1',
      install : true,
      install_dir : get_option('mandir') / 'man1',
    )
  endforeach
endforeach

hoffman3_manpages = [
  'hoffman3-build',
  'hoffman3-bundle',
  'hoffman3-config',
  'hoffman3-config-check',
  'hoffman3-config-show',
  'hoffman3-copy',
  'hoffman3-daemon',
  'hoffman3-derivation-add',
  'hoffman3-derivation',
  'hoffman3-derivation-show',
  'hoffman3-develop',
  'hoffman3-edit',
  'hoffman3-env-shell',
  'hoffman3-eval',
  'hoffman3-grass-archive',
  'hoffman3-grass-check',
  'hoffman3-grass-clone',
  'hoffman3-grass-info',
  'hoffman3-grass-init',
  'hoffman3-grass-lock',
  'hoffman3-grass',
  'hoffman3-grass-metadata',
  'hoffman3-grass-new',
  'hoffman3-grass-prefetch',
  'hoffman3-grass-show',
  'hoffman3-grass-update',
  'hoffman3-fmt',
  'hoffman3-hash-file',
  'hoffman3-hash',
  'hoffman3-hash-convert',
  'hoffman3-hash-path',
  'hoffman3-hash-to-base16',
  'hoffman3-hash-to-base32',
  'hoffman3-hash-to-base64',
  'hoffman3-hash-to-sri',
  'hoffman3-help',
  'hoffman3-help-stores',
  'hoffman3-key-convert-secret-to-public',
  'hoffman3-key-generate-secret',
  'hoffman3-key',
  'hoffman3-log',
  'hoffman3-nar-cat',
  'hoffman3-nar-dump-path',
  'hoffman3-nar-ls',
  'hoffman3-nar-pack',
  'hoffman3-nar',
  'hoffman3-path-info',
  'hoffman3-print-dev-env',
  'hoffman3-profile',
  'hoffman3-profile-add',
  'hoffman3-profile-diff-closures',
  'hoffman3-profile-history',
  'hoffman3-profile-list',
  'hoffman3-profile-remove',
  'hoffman3-profile-rollback',
  'hoffman3-profile-upgrade',
  'hoffman3-profile-wipe-history',
  'hoffman3-realisation-info',
  'hoffman3-realisation',
  'hoffman3-registry-add',
  'hoffman3-registry-list',
  'hoffman3-registry',
  'hoffman3-registry-pin',
  'hoffman3-registry-remove',
  'hoffman3-repl',
  'hoffman3-run',
  'hoffman3-search',
  'hoffman3-store-add',
  'hoffman3-store-add-file',
  'hoffman3-store-add-path',
  'hoffman3-store-cat',
  'hoffman3-store-copy-log',
  'hoffman3-store-copy-sigs',
  'hoffman3-store-delete',
  'hoffman3-store-diff-closures',
  'hoffman3-store-dump-path',
  'hoffman3-store-gc',
  'hoffman3-store-info',
  'hoffman3-store-ls',
  'hoffman3-store-make-content-addressed',
  'hoffman3-store',
  'hoffman3-store-optimise',
  'hoffman3-store-path-from-hash-part',
  'hoffman3-store-prefetch-file',
  'hoffman3-store-repair',
  'hoffman3-store-sign',
  'hoffman3-store-verify',
  'hoffman3-upgrade-hoffman',
  'hoffman3-why-depends',
  'hoffman',
]

foreach page : hoffman3_manpages
  section = '1'
  custom_target(
    command : [
      bash,
      '@INPUT0@',
      # Note: no --out-no-smarty flag (original behavior)
      page,
      section,
      meson.current_source_dir() / 'source',
      meson.current_build_dir() / 'source',
      doc_url,
      meson.current_build_dir() / 'source/command-ref/new-cli/@0@.md'.format(
        page,
      ),
      '@OUTPUT@',
    ],
    input : [
      files('./render-manpage.sh'),
      files('./expand-includes.py'),
      hoffman3_cli_files,
      hoffman_input,
    ],
    output : page + '.1',
    install : true,
    install_dir : get_option('mandir') / 'man1',
  )
endforeach

hoffman_manpages = [
  [ 'hoffman-env', 1 ],
  [ 'hoffman-store', 1 ],
  [ 'hoffman-build', 1 ],
  [ 'hoffman-shell', 1 ],
  [ 'hoffman-instantiate', 1 ],
  [ 'hoffman-collect-garbage', 1 ],
  [ 'hoffman-prefetch-url', 1 ],
  [ 'hoffman-channel', 1 ],
  [ 'hoffman-hash', 1 ],
  [ 'hoffman-copy-closure', 1 ],
  [
    'hoffman.conf',
    5,
    conf_file_md.full_path(),
    [ conf_file_md, experimental_features_shortlist_md ],
  ],
  [ 'hoffman-daemon', 8 ],
  [ 'hoffman-profiles', 5, 'files/profiles.md' ],
]

foreach entry : hoffman_manpages
  title = entry[0]
  # hoffman.conf.5 and hoffman-profiles.5 are based off of conf-file.md and files/profiles.md,
  # rather than a stem identical to its mdbook source.
  # Therefore we use an optional third element of this array to override the name pattern
  md_file = entry.get(2, title + '.md')
  section = entry[1].to_string()
  input_file = meson.current_source_dir() / 'source/command-ref' / md_file

  custom_target(
    command : [
      bash,
      '@INPUT0@',
      # Note: no --out-no-smarty flag (original behavior)
      title,
      section,
      meson.current_source_dir() / 'source',
      meson.current_build_dir() / 'source',
      doc_url,
      input_file,
      '@OUTPUT@',
    ],
    input : [
      files('./render-manpage.sh'),
      files('./expand-includes.py'),
      entry.get(3, []),
      hoffman_input,
    ],
    output : '@0@.@1@'.format(entry[0], entry[1]),
    install : true,
    install_dir : get_option('mandir') / 'man@0@'.format(entry[1]),
  )
endforeach
