#include "hoffman/cmd/command.hh"

namespace hoffman {

struct CmdStore : HoffmanMultiCommand
{
    CmdStore()
        : HoffmanMultiCommand("store", RegisterCommand::getCommandsFor({"store"}))
    {
        aliases = {
            {"ping", {AliasStatus::Deprecated, {"info"}}},
        };
    }

    std::string description() override
    {
        return "manipulate a Hoffman store";
    }

    Category category() override
    {
        return catUtility;
    }
};

static auto rCmdStore = registerCommand<CmdStore>("store");

} // namespace hoffman
