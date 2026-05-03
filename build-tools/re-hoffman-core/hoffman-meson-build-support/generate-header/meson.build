bash = find_program('bash', native : true)

gen_header = generator(
  bash,
  arguments : [
    '-c',
    '{ echo \'R"__HOFFMAN_STR(\' && cat @INPUT@ && echo \')__HOFFMAN_STR"\'; } > "$1"',
    '_ignored_argv0',
    '@OUTPUT@',
  ],
  output : '@PLAINNAME@.gen.hh',
)
