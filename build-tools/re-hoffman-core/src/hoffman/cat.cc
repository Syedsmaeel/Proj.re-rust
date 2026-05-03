#include "hoffman/cmd/command.hh"
#include "hoffman/store/store-api.hh"
#include "hoffman/util/archive.hh"
#include "hoffman/util/serialise.hh"
#include "hoffman/util/source-accessor.hh"

#include <nlohmann/json.hpp>

namespace hoffman {

struct MixCat : virtual Args
{
    void cat(ref<SourceAccessor> accessor, CanonPath path)
    {
        auto st = accessor->lstat(path);
        if (st.type != SourceAccessor::Type::tRegular)
            throw Error("path '%1%' is not a regular file", path.abs());
        logger->stop();

        FdSink output{getStandardOutput()};
        accessor->readFile(path, output);
        output.flush();
    }
};

struct CmdCatStore : StoreCommand, MixCat
{
    std::string path;

    CmdCatStore()
    {
        expectArgs({.label = "path", .handler = {&path}, .completer = completePath});
    }

    std::string description() override
    {
        return "print the contents of a file in the Hoffman store on stdout";
    }

    std::string doc() override
    {
        return
#include "store-cat.md"
            ;
    }

    void run(ref<Store> store) override
    {
        auto [storePath, rest] = store->toStorePath(path);
        cat(store->requireStoreObjectAccessor(storePath), rest);
    }
};

struct CmdCatNar : StoreCommand, MixCat
{
    std::filesystem::path narPath;

    std::string path;

    CmdCatNar()
    {
        expectArgs({.label = "nar", .handler = {&narPath}, .completer = completePath});
        expectArg("path", &path);
    }

    std::string description() override
    {
        return "print the contents of a file inside a NAR file on stdout";
    }

    std::string doc() override
    {
        return
#include "nar-cat.md"
            ;
    }

    void run(ref<Store> store) override
    {
        auto fd = openFileReadonly(narPath);
        if (!fd)
            throw NativeSysError("opening NAR file %s", PathFmt(narPath));
        auto source = FdSource{fd.get()};

        struct CatRegularFileSink : NullFileSystemObjectSink
        {
            CanonPath neededPath = CanonPath::root;
            bool found = false;

            void createRegularFile(const CanonPath & path, fun<void(CreateRegularFileSink &)> crf) override
            {
                struct : CreateRegularFileSink, FdSink
                {
                    void isExecutable() override {}
                } crfSink;

                crfSink.fd = INVALID_DESCRIPTOR;

                if (path == neededPath) {
                    logger->stop();
                    crfSink.skipContents = false;
                    crfSink.fd = getStandardOutput();
                    found = true;
                } else {
                    crfSink.skipContents = true;
                }

                crf(crfSink);
            }
        } sink;

        sink.neededPath = CanonPath(path);
        /* NOTE: We still parse the whole file to validate that it's a correct NAR. */
        parseDump(sink, source);

        if (!sink.found)
            throw Error("NAR does not contain regular file '%1%'", path);
    }
};

static auto rCmdCatStore = registerCommand2<CmdCatStore>({"store", "cat"});
static auto rCmdCatNar = registerCommand2<CmdCatNar>({"nar", "cat"});

} // namespace hoffman
