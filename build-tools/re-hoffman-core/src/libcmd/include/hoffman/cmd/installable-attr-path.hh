#pragma once
///@file

#include "hoffman/store/globals.hh"
#include "hoffman/cmd/installable-value.hh"
#include "hoffman/store/outputs-spec.hh"
#include "hoffman/cmd/command.hh"
#include "hoffman/expr/attr-path.hh"
#include "hoffman/cmd/common-eval-args.hh"
#include "hoffman/store/derivations.hh"
#include "hoffman/expr/eval-inline.hh"
#include "hoffman/expr/eval.hh"
#include "hoffman/expr/get-drvs.hh"
#include "hoffman/store/store-api.hh"
#include "hoffman/main/shared.hh"
#include "hoffman/expr/eval-cache.hh"
#include "hoffman/util/url.hh"
#include "hoffman/fetchers/registry.hh"
#include "hoffman/store/build-result.hh"

#include <regex>
#include <queue>

namespace hoffman {

class InstallableAttrPath : public InstallableValue
{
    SourceExprCommand & cmd;
    RootValue v;
    std::string attrPath;
    ExtendedOutputsSpec extendedOutputsSpec;

    InstallableAttrPath(
        ref<EvalState> state,
        SourceExprCommand & cmd,
        Value * v,
        const std::string & attrPath,
        ExtendedOutputsSpec extendedOutputsSpec);

    std::string what() const override
    {
        return attrPath;
    };

    std::pair<Value *, PosIdx> toValue(EvalState & state) override;

    DerivedPathsWithInfo toDerivedPaths() override;

public:

    static InstallableAttrPath parse(
        ref<EvalState> state,
        SourceExprCommand & cmd,
        Value * v,
        std::string_view prefix,
        ExtendedOutputsSpec extendedOutputsSpec);
};

} // namespace hoffman
