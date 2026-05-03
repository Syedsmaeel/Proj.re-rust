#pragma once
/// @file

#include "hoffman/store/binary-cache-store.hh"

namespace hoffman {

struct LocalBinaryCacheStoreConfig : std::enable_shared_from_this<LocalBinaryCacheStoreConfig>,
                                     virtual Store::Config,
                                     BinaryCacheStoreConfig
{
private:
    void anchor() override;

public:
    LocalBinaryCacheStoreConfig(const Params & params)
        : StoreConfig(params, FilePathType::Uhoffman)
        , BinaryCacheStoreConfig(params)
    {
    }

    /**
     * @param binaryCacheDir `file://` is a short-hand for `file:///`
     * for now.
     */
    LocalBinaryCacheStoreConfig(const std::filesystem::path & binaryCacheDir, const Params & params);

    std::filesystem::path binaryCacheDir;

    static const std::string name()
    {
        return "Local Binary Cache Store";
    }

    static StringSet uriSchemes();

    static std::string doc();

    ref<Store> openStore() const override;

    StoreReference getReference() const override;
};

} // namespace hoffman
