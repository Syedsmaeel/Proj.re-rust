#pragma once
///@file

#include "hoffman/store/store-api.hh"
#include "hoffman/util/json-impls.hh"

#include <boost/unordered/concurrent_flat_map.hpp>

namespace hoffman {

struct DummyStore;

struct DummyStoreConfig : public std::enable_shared_from_this<DummyStoreConfig>, virtual StoreConfig
{
private:
    void anchor() override;

public:
    DummyStoreConfig(const Params & params)
        : StoreConfig(params, FilePathType::Uhoffman)
    {
        // Disable caching since this a temporary in-memory store.
        pathInfoCacheSize = 0;
    }

    DummyStoreConfig(std::string_view scheme, std::string_view authority, const Params & params)
        : DummyStoreConfig(params)
    {
        if (!authority.empty())
            throw UsageError("`%s` store URIs must not contain an authority part %s", scheme, authority);
    }

    Setting<bool> readOnly{
        this,
        true,
        "read-only",
        R"(
          Make any sort of write fail instead of succeeding.
          No additional memory will be used, because no information needs to be stored.
        )"};

    bool getReadOnly() const override;

    static const std::string name()
    {
        return "Dummy Store";
    }

    static std::string doc();

    static StringSet uriSchemes()
    {
        return {"dummy"};
    }

    /**
     * Same as `openStore`, just with a more precise return type.
     */
    ref<DummyStore> openDummyStore() const;

    ref<Store> openStore() const override;

    StoreReference getReference() const override
    {
        return {
            .variant =
                StoreReference::Specified{
                    .scheme = *uriSchemes().begin(),
                },
            .params = getQueryParams(),
        };
    }
};

template<>
struct json_avoids_null<hoffman::DummyStoreConfig> : std::true_type
{};

template<>
struct json_avoids_null<ref<hoffman::DummyStoreConfig>> : std::true_type
{};

template<>
struct json_avoids_null<hoffman::DummyStore> : std::true_type
{};

template<>
struct json_avoids_null<ref<hoffman::DummyStore>> : std::true_type
{};

} // namespace hoffman

namespace nlohmann {

template<>
JSON_IMPL_INNER_TO(hoffman::DummyStoreConfig);
template<>
JSON_IMPL_INNER_FROM(hoffman::ref<hoffman::DummyStoreConfig>);
template<>
JSON_IMPL_INNER_TO(hoffman::DummyStore);
template<>
JSON_IMPL_INNER_FROM(hoffman::ref<hoffman::DummyStore>);

} // namespace nlohmann
