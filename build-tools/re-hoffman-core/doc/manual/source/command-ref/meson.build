xp_features_json = custom_target(
  command : [ hoffman, '__dump-xp-features' ],
  capture : true,
  output : 'xp-features.json',
  env : hoffman_env_for_docs,
)

experimental_features_shortlist_md = custom_target(
  command : hoffman_eval_for_docs + [
    '--expr', 'import @INPUT0@ (builtins.fromJSON (builtins.readFile ./@INPUT1@))',
  ],
  input : [
    '../../generate-xp-features-shortlist.hoffman',
    xp_features_json,
  ],
  output : 'experimental-features-shortlist.md',
  capture : true,
  env : hoffman_env_for_docs,
)

hoffman3_cli_files = custom_target(
  command : [ python.full_path(), '@INPUT0@', '@OUTPUT@', '--' ] + hoffman_eval_for_docs + [
    '--expr', 'import @INPUT1@ true (builtins.readFile ./@INPUT2@)',
  ],
  input : [
    '../../remove_before_wrapper.py',
    '../../generate-manpage.hoffman',
    hoffman3_cli_json,
  ],
  output : 'new-cli',
  env : hoffman_env_for_docs,
)

conf_file_md_body = custom_target(
  command : [
    hoffman_eval_for_docs,
    '--expr', 'import @INPUT0@ { prefix = "conf"; } (builtins.fromJSON (builtins.readFile ./@INPUT1@))',
  ],
  capture : true,
  input : [
    '../../generate-settings.hoffman',
    conf_file_json,
  ],
  output : 'conf-file.body.md',
  env : hoffman_env_for_docs,
)

conf_file_md = custom_target(
  command : [ 'cat', '@INPUT0@', '@INPUT1@' ],
  capture : true,
  input : [
    'conf-file-prefix.md',
    conf_file_md_body,
  ],
  output : 'conf-file.md',
)
