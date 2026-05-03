#include <vector>

#include "hoffman/flake/settings.hh"
#include "hoffman/flake/flake-primops.hh"
#include "hoffman/expr/eval-settings.hh"
#include "hoffman/expr/eval.hh"

namespace hoffman::flake {

Settings::Settings() {}

void Settings::configureEvalSettings(hoffman::EvalSettings & evalSettings) const
{
    evalSettings.extraPrimOps.emplace_back(primops::getFlake(*this));
    evalSettings.extraPrimOps.emplace_back(primops::parseFlakeRef);
    evalSettings.extraPrimOps.emplace_back(primops::flakeRefToString);
}

} // namespace hoffman::flake
