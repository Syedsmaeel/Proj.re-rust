#pragma once

#include "hoffman/cmd/command.hh"
#include "hoffman/cmd/installable-flake.hh"
#include "hoffman/flake/flake.hh"

namespace hoffman {

class FlakeCommand : virtual Args, public MixFlakeOptions
{
protected:
    std::string flakeUrl = ".";

public:

    FlakeCommand();

    FlakeRef getFlakeRef();

    flake::LockedFlake lockFlake();

    std::vector<FlakeRef> getFlakeRefsForCompletion() override;
};

} // namespace hoffman
