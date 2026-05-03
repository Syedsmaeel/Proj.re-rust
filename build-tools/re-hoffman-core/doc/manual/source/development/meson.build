experimental_feature_descriptions_md = custom_target(
  command : hoffman_eval_for_docs + [
    '--expr', 'import @INPUT0@ (builtins.fromJSON (builtins.readFile @INPUT1@))',
  ],
  input : [
    '../../generate-xp-features.hoffman',
    xp_features_json,
  ],
  capture : true,
  env : hoffman_env_for_docs,
  output : 'experimental-feature-descriptions.md',
)
