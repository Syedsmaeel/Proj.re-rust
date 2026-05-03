#include "hoffman/cmd/command.hh"
#include "hoffman/store/store-api.hh"
#include "hoffman/util/archive.hh"
#include "hoffman/util/terminal.hh"

namespace hoffman {

static FdSink getNarSink()
{
    auto fd = getStandardOutput();
    if (isTTY(fd))
        throw UsageError("refusing to write NAR to a terminal");
    return FdSink(std::move(fd));
}

struct CmdDumpPath : StorePathCommand
{
    std::string description() override
    {
        return "serialise a store path to stdout in NAR format";
    }

    std::string doc() override
    {
        return
#include "store-dump-path.md"
            ;
    }

    void run(ref<Store> store, const StorePath & storePath) override
    {
        auto sink = getNarSink();
        store->narFromPath(storePath, sink);
        sink.flush();
    }
};

static auto rDumpPath = registerCommand2<CmdDumpPath>({"store", "dump-path"});

struct CmdDumpPath2 : Command
{
    std::filesystem::path path;

    CmdDumpPath2()
    {
        expectArgs({.label = "path", .handler = {&path}, .completer = completePath});
    }

    std::string description() override
    {
        return "serialise a path to stdout in NAR format";
    }

    std::string doc() override
    {
        return
#include "nar-dump-path.md"
            ;
    }

    void run() override
    {
        auto sink = getNarSink();
        dumpPath(path, sink);
        sink.flush();
    }
};

struct CmdNarDumpPath : CmdDumpPath2
{
    void run() override
    {
        warn("'hoffman nar dump-path' is a deprecated alias for 'hoffman nar pack'");
        CmdDumpPath2::run();
    }
};

static auto rCmdNarPack = registerCommand2<CmdDumpPath2>({"nar", "pack"});
static auto rCmdNarDumpPath = registerCommand2<CmdNarDumpPath>({"nar", "dump-path"});

} // namespace hoffman
