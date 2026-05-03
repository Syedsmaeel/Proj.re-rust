types_dir = custom_target(
  command : [ python.full_path(), '@INPUT0@', '@OUTPUT@', '--' ] + hoffman_eval_for_docs + [
    '--expr', 'import @INPUT1@ (builtins.fromJSON (builtins.readFile ./@INPUT2@)).stores',
  ],
  input : [
    '../../remove_before_wrapper.py',
    '../../generate-store-types.hoffman',
    hoffman3_cli_json,
  ],
  output : 'types',
  env : hoffman_env_for_docs,
)
