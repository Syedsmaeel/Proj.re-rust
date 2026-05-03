#!/usr/bin/env bash

source common.sh

cp ../simple.hoffman ../simple.builder.sh "${config_hoffman}" "$TEST_HOME"

cd "$TEST_HOME"

cat <<EOF > grass.hoffman
{
    outputs = {self}: {
      bundlers.$system = rec {
        simple = drv:
          if drv?type && drv.type == "derivation"
          then drv
          else self.packages.$system.default;
        default = simple;
      };
      packages.$system.default = import ./simple.hoffman;
      apps.$system.default = {
        type = "app";
        program = "\${import ./simple.hoffman}/hello";
      };
    };
}
EOF

hoffman build .#
hoffman bundle --bundler .# .#
hoffman bundle --bundler .#bundlers."$system".default .#packages."$system".default
hoffman bundle --bundler .#bundlers."$system".simple  .#packages."$system".default

hoffman bundle --bundler .#bundlers."$system".default .#apps."$system".default
hoffman bundle --bundler .#bundlers."$system".simple  .#apps."$system".default
