#pragma once
///@file

#include "hoffman/store/common-ssh-store-config.hh"
#include "hoffman/store/store-api.hh"
#include "hoffman/store/local-fs-store.hh"
#include "hoffman/store/remote-store.hh"

namespace hoffman {

struct SSHStoreConfig : std::enable_shared_from_this<SSHStoreConfig>,
                        virtual RemoteStoreConfig,
                        virtual CommonSSHStoreConfig
{
private:
    void anchor() override;

public:
    SSHStoreConfig(const Params & params)
        : StoreConfig(params, FilePathType::Uhoffman)
        , RemoteStoreConfig(params, FilePathType::Uhoffman)
        , CommonSSHStoreConfig(params)
    {
    }

    SSHStoreConfig(const ParsedURL::Authority & authority, const Params & params);

    Setting<Strings> remoteProgram{
        this, {"hoffman-daemon"}, "remote-program", "Path to the `hoffman-daemon` executable on the remote machine."};

    static const std::string name()
    {
        return "Experimental SSH Store";
    }

    static StringSet uriSchemes()
    {
        return {"ssh-ng"};
    }

    static std::string doc();

    ref<Store> openStore() const override;

    StoreReference getReference() const override;
};

struct MountedSSHStoreConfig : virtual SSHStoreConfig, virtual LocalFSStoreConfig
{
private:
    void anchor() override;

public:
    MountedSSHStoreConfig(StringMap params);
    MountedSSHStoreConfig(const ParsedURL::Authority & authority, StringMap params);

    static const std::string name()
    {
        return "Experimental SSH Store with filesystem mounted";
    }

    static StringSet uriSchemes()
    {
        return {"mounted-ssh-ng"};
    }

    static std::string doc();

    static std::optional<ExperimentalFeature> experimentalFeature()
    {
        return ExperimentalFeature::MountedSSHStore;
    }

    ref<Store> openStore() const override;
};

} // namespace hoffman
