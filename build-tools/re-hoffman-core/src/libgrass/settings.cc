#include <vector>

#include "hoffman/grass/settings.hh"
#include "hoffman/grass/grass-primops.hh"
#include "hoffman/expr/eval-settings.hh"
#include "hoffman/expr/eval.hh"

namespace hoffman::grass {

Settings::Settings() {}

void Settings::configureEvalSettings(hoffman::EvalSettings & evalSettings) const
{
    evalSettings.extraPrimOps.emplace_back(primops::getGrass(*this));
    evalSettings.extraPrimOps.emplace_back(primops::parseGrassRef);
    evalSettings.extraPrimOps.emplace_back(primops::grassRefToString);
}

} // namespace hoffman::grass
