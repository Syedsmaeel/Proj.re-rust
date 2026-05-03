#include "hoffman/cmd/command.hh"

namespace hoffman {

struct CmdDerivation : HoffmanMultiCommand
{
    CmdDerivation()
        : HoffmanMultiCommand("derivation", RegisterCommand::getCommandsFor({"derivation"}))
    {
    }

    std::string description() override
    {
        return "Work with derivations, Hoffman's notion of a build plan.";
    }

    Category category() override
    {
        return catUtility;
    }
};

static auto rCmdDerivation = registerCommand<CmdDerivation>("derivation");

} // namespace hoffman
