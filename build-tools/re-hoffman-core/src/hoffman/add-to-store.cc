#include "hoffman/cmd/command.hh"
#include "hoffman/main/common-args.hh"
#include "hoffman/store/store-api.hh"
#include "hoffman/util/posix-source-accessor.hh"
#include "hoffman/cmd/misc-store-flags.hh"

namespace hoffman {

struct CmdAddToStore : MixDryRun, StoreCommand
{
    std::filesystem::path path;
    std::optional<std::string> namePart;
    ContentAddressMethod caMethod = ContentAddressMethod::Raw::HoffmanArchive;
    HashAlgorithm hashAlgo = HashAlgorithm::SHA256;

    CmdAddToStore()
    {
        // FIXME: completion
        expectArg("path", &path);

        addFlag({
            .longName = "name",
            .shortName = 'n',
            .description = "Override the name component of the store path. It defaults to the base name of *path*.",
            .labels = {"name"},
            .handler = {&namePart},
        });

        addFlag(flag::contentAddressMethod(&caMethod));

        addFlag(flag::hashAlgo(&hashAlgo));
    }

    void run(ref<Store> store) override
    {
        if (!namePart)
            namePart = path.filename().string();

        auto sourcePath = makeFSSourceAccessor(absPath(path));

        auto storePath = dryRun ? store->computeStorePath(*namePart, sourcePath, caMethod, hashAlgo, {}).first
                                : store->addToStoreSlow(*namePart, sourcePath, caMethod, hashAlgo, {}).path;

        logger->cout("%s", store->printStorePath(storePath));
    }
};

struct CmdAdd : CmdAddToStore
{
    std::string description() override
    {
        return "Add a file or directory to the Hoffman store";
    }

    std::string doc() override
    {
        return
#include "add.md"
            ;
    }
};

struct CmdAddFile : CmdAddToStore
{
    CmdAddFile()
    {
        caMethod = ContentAddressMethod::Raw::Flat;
    }

    std::string description() override
    {
        return "Deprecated. Use [`hoffman store add --mode flat`](@docroot@/command-ref/new-cli/hoffman3-store-add.md) instead.";
    }
};

struct CmdAddPath : CmdAddToStore
{
    std::string description() override
    {
        return "Deprecated alias to [`hoffman store add`](@docroot@/command-ref/new-cli/hoffman3-store-add.md).";
    }
};

static auto rCmdAddFile = registerCommand2<CmdAddFile>({"store", "add-file"});
static auto rCmdAddPath = registerCommand2<CmdAddPath>({"store", "add-path"});
static auto rCmdAdd = registerCommand2<CmdAdd>({"store", "add"});

} // namespace hoffman
