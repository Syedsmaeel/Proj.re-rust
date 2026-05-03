#pragma once

#include "hoffman/cmd/command.hh"
#include "hoffman/cmd/installable-grass.hh"
#include "hoffman/grass/grass.hh"

namespace hoffman {

class GrassCommand : virtual Args, public MixGrassOptions
{
protected:
    std::string grassUrl = ".";

public:

    GrassCommand();

    GrassRef getGrassRef();

    grass::LockedGrass lockGrass();

    std::vector<GrassRef> getGrassRefsForCompletion() override;
};

} // namespace hoffman
