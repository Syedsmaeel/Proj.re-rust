#include <cstring>
#include <span>

#include "hoffman_api_store.h"
#include "hoffman_api_store_internal.h"
#include "hoffman_api_util.h"
#include "hoffman_api_util_internal.h"

#include "hoffman/store/path.hh"
#include "hoffman/store/store-api.hh"
#include "hoffman/store/store-open.hh"
#include "hoffman/store/store-reference.hh"
#include "hoffman/store/build-result.hh"
#include "hoffman/store/local-fs-store.hh"
#include "hoffman/util/base-hoffman-32.hh"

#include "hoffman/store/globals.hh"

extern "C" {

hoffman_err hoffman_libstore_init(hoffman_c_context * context)
{
    if (context)
        context->last_err_code = HOFFMAN_OK;
    try {
        hoffman::initLibStore();
    }
    HOFFMANC_CATCH_ERRS
}

hoffman_err hoffman_libstore_init_no_load_config(hoffman_c_context * context)
{
    if (context)
        context->last_err_code = HOFFMAN_OK;
    try {
        hoffman::initLibStore(false);
    }
    HOFFMANC_CATCH_ERRS
}

Store * hoffman_store_open(hoffman_c_context * context, const char * uri, const char *** params)
{
    if (context)
        context->last_err_code = HOFFMAN_OK;
    try {
        std::string uri_str = uri ? uri : "";

        if (uri_str.empty())
            return new Store{hoffman::openStore()};

        auto storeRef = hoffman::StoreReference::parse(uri_str);

        if (params) {
            for (size_t i = 0; params[i] != nullptr; i++) {
                storeRef.params[params[i][0]] = params[i][1];
            }
        }
        return new Store{hoffman::openStore(std::move(storeRef))};
    }
    HOFFMANC_CATCH_ERRS_NULL
}

void hoffman_store_free(Store * store)
{
    delete store;
}

hoffman_err hoffman_store_get_uri(hoffman_c_context * context, Store * store, hoffman_get_string_callback callback, void * user_data)
{
    if (context)
        context->last_err_code = HOFFMAN_OK;
    try {
        auto res = store->ptr->config.getReference().render(/*withParams=*/true);
        return call_hoffman_get_string_callback(res, callback, user_data);
    }
    HOFFMANC_CATCH_ERRS
}

hoffman_err
hoffman_store_get_storedir(hoffman_c_context * context, Store * store, hoffman_get_string_callback callback, void * user_data)
{
    if (context)
        context->last_err_code = HOFFMAN_OK;
    try {
        return call_hoffman_get_string_callback(store->ptr->storeDir, callback, user_data);
    }
    HOFFMANC_CATCH_ERRS
}

hoffman_err
hoffman_store_get_version(hoffman_c_context * context, Store * store, hoffman_get_string_callback callback, void * user_data)
{
    if (context)
        context->last_err_code = HOFFMAN_OK;
    try {
        auto res = store->ptr->getVersion();
        return call_hoffman_get_string_callback(res.value_or(""), callback, user_data);
    }
    HOFFMANC_CATCH_ERRS
}

bool hoffman_store_is_valid_path(hoffman_c_context * context, Store * store, const StorePath * path)
{
    if (context)
        context->last_err_code = HOFFMAN_OK;
    try {
        return store->ptr->isValidPath(path->path);
    }
    HOFFMANC_CATCH_ERRS_RES(false);
}

hoffman_err hoffman_store_real_path(
    hoffman_c_context * context, Store * store, StorePath * path, hoffman_get_string_callback callback, void * user_data)
{
    if (context)
        context->last_err_code = HOFFMAN_OK;
    try {
        auto store2 = store->ptr.dynamic_pointer_cast<hoffman::LocalFSStore>();
        auto res = store2 ? store2->toRealPath(path->path).string() : store->ptr->printStorePath(path->path);
        return call_hoffman_get_string_callback(res, callback, user_data);
    }
    HOFFMANC_CATCH_ERRS
}

StorePath * hoffman_store_parse_path(hoffman_c_context * context, Store * store, const char * path)
{
    if (context)
        context->last_err_code = HOFFMAN_OK;
    try {
        hoffman::StorePath s = store->ptr->parseStorePath(path);
        return new StorePath{std::move(s)};
    }
    HOFFMANC_CATCH_ERRS_NULL
}

hoffman_err hoffman_store_get_fs_closure(
    hoffman_c_context * context,
    Store * store,
    const StorePath * store_path,
    bool flip_direction,
    bool include_outputs,
    bool include_derivers,
    void * userdata,
    void (*callback)(hoffman_c_context * context, void * userdata, const StorePath * store_path))
{
    if (context)
        context->last_err_code = HOFFMAN_OK;
    try {
        const auto hoffmanStore = store->ptr;

        hoffman::StorePathSet set;
        hoffmanStore->computeFSClosure(store_path->path, set, flip_direction, include_outputs, include_derivers);

        if (callback) {
            for (const auto & path : set) {
                const StorePath tmp{path};
                callback(context, userdata, &tmp);
                if (context && context->last_err_code != HOFFMAN_OK)
                    return context->last_err_code;
            }
        }
    }
    HOFFMANC_CATCH_ERRS
}

hoffman_err hoffman_store_realise(
    hoffman_c_context * context,
    Store * store,
    StorePath * path,
    void * userdata,
    void (*callback)(void * userdata, const char *, const StorePath *))
{
    if (context)
        context->last_err_code = HOFFMAN_OK;
    try {

        const std::vector<hoffman::DerivedPath> paths{hoffman::DerivedPath::Built{
            .drvPath = hoffman::makeConstantStorePathRef(path->path), .outputs = hoffman::OutputsSpec::All{}}};

        const auto hoffmanStore = store->ptr;
        auto results = hoffmanStore->buildPathsWithResults(paths, hoffman::bmNormal, hoffmanStore);

        assert(results.size() == 1);

        // Check if any builds failed
        for (auto & result : results)
            result.tryThrowBuildError();

        if (callback) {
            for (const auto & result : results) {
                if (auto * success = result.tryGetSuccess()) {
                    for (const auto & [outputName, realisation] : success->builtOutputs) {
                        StorePath p{realisation.outPath};
                        callback(userdata, outputName.c_str(), &p);
                    }
                }
            }
        }
    }
    HOFFMANC_CATCH_ERRS
}

void hoffman_store_path_name(const StorePath * store_path, hoffman_get_string_callback callback, void * user_data)
{
    std::string_view name = store_path->path.name();
    callback(name.data(), name.size(), user_data);
}

void hoffman_store_path_free(StorePath * sp)
{
    delete sp;
}

void hoffman_derivation_free(hoffman_derivation * drv)
{
    delete drv;
}

StorePath * hoffman_store_path_clone(const StorePath * p)
{
    try {
        return new StorePath{p->path};
    } catch (...) {
        return nullptr;
    }
}

} // extern "C"

template<size_t S>
static auto to_cpp_array(const uint8_t (&r)[S])
{
    return reinterpret_cast<const std::array<std::byte, S> &>(r);
}

extern "C" {

hoffman_err
hoffman_store_path_hash(hoffman_c_context * context, const StorePath * store_path, hoffman_store_path_hash_part * hash_part_out)
{
    try {
        auto hashPart = store_path->path.hashPart();
        // Decode from Hoffman32 (base32) encoding to raw bytes
        auto decoded = hoffman::BaseHoffman32::decode(hashPart);

        assert(decoded.size() == sizeof(hash_part_out->bytes));
        std::memcpy(hash_part_out->bytes, decoded.data(), sizeof(hash_part_out->bytes));
        return HOFFMAN_OK;
    }
    HOFFMANC_CATCH_ERRS
}

StorePath * hoffman_store_create_from_parts(
    hoffman_c_context * context, const hoffman_store_path_hash_part * hash, const char * name, size_t name_len)
{
    if (context)
        context->last_err_code = HOFFMAN_OK;
    try {
        // Encode the 20 raw bytes to Hoffman32 (base32) format
        auto hashStr = hoffman::BaseHoffman32::encode(std::span<const std::byte>{to_cpp_array(hash->bytes)});

        // Construct the store path basename: <hash>-<name>
        std::string baseName;
        baseName += hashStr;
        baseName += "-";
        baseName += std::string_view{name, name_len};

        return new StorePath{hoffman::StorePath(std::move(baseName))};
    }
    HOFFMANC_CATCH_ERRS_NULL
}

hoffman_derivation * hoffman_derivation_clone(const hoffman_derivation * d)
{
    try {
        return new hoffman_derivation{d->drv};
    } catch (...) {
        return nullptr;
    }
}

hoffman_derivation * hoffman_derivation_from_json(hoffman_c_context * context, Store * store, const char * json)
{
    if (context)
        context->last_err_code = HOFFMAN_OK;
    try {
        return new hoffman_derivation{hoffman::Derivation::parseJsonAndValidate(*store->ptr, nlohmann::json::parse(json))};
    }
    HOFFMANC_CATCH_ERRS_NULL
}

hoffman_err hoffman_derivation_to_json(
    hoffman_c_context * context, const hoffman_derivation * drv, hoffman_get_string_callback callback, void * userdata)
{
    if (context)
        context->last_err_code = HOFFMAN_OK;
    try {
        auto result = static_cast<nlohmann::json>(drv->drv).dump();
        if (callback) {
            callback(result.data(), result.size(), userdata);
        }
    }
    HOFFMANC_CATCH_ERRS
}

StorePath * hoffman_add_derivation(hoffman_c_context * context, Store * store, hoffman_derivation * derivation)
{
    if (context)
        context->last_err_code = HOFFMAN_OK;
    try {
        /* Quite dubious that users would want this to silently suceed
           without actually writing the derivation if this setting is
           set, but it was that way already, so we are doing this for
           back-compat for now. */
        auto ret = hoffman::settings.readOnlyMode ? hoffman::computeStorePath(*store->ptr, derivation->drv)
                                              : store->ptr->writeDerivation(derivation->drv, hoffman::NoRepair);

        return new StorePath{ret};
    }
    HOFFMANC_CATCH_ERRS_NULL
}

hoffman_err hoffman_store_copy_closure(hoffman_c_context * context, Store * srcStore, Store * dstStore, StorePath * path)
{
    if (context)
        context->last_err_code = HOFFMAN_OK;
    try {
        hoffman::RealisedPath::Set paths;
        paths.insert(path->path);
        hoffman::copyClosure(*srcStore->ptr, *dstStore->ptr, paths);
    }
    HOFFMANC_CATCH_ERRS
}

hoffman_derivation * hoffman_store_drv_from_store_path(hoffman_c_context * context, Store * store, const StorePath * path)
{
    if (context)
        context->last_err_code = HOFFMAN_OK;
    try {
        return new hoffman_derivation{store->ptr->derivationFromPath(path->path)};
    }
    HOFFMANC_CATCH_ERRS_NULL
}

StorePath * hoffman_store_query_path_from_hash_part(hoffman_c_context * context, Store * store, const char * hash)
{
    if (context)
        context->last_err_code = HOFFMAN_OK;
    try {
        std::optional<hoffman::StorePath> s = store->ptr->queryPathFromHashPart(hash);

        if (!s.has_value()) {
            return nullptr;
        }

        return new StorePath{std::move(s.value())};
    }
    HOFFMANC_CATCH_ERRS_NULL
}

hoffman_err hoffman_store_copy_path(
    hoffman_c_context * context, Store * srcStore, Store * dstStore, const StorePath * path, bool repair, bool checkSigs)
{
    if (context)
        context->last_err_code = HOFFMAN_OK;
    try {
        if (srcStore == nullptr)
            return hoffman_set_err_msg(context, HOFFMAN_ERR_UNKNOWN, "Source store is null");

        if (dstStore == nullptr)
            return hoffman_set_err_msg(context, HOFFMAN_ERR_UNKNOWN, "Destination store is null");

        if (path == nullptr)
            return hoffman_set_err_msg(context, HOFFMAN_ERR_UNKNOWN, "Store path is null");

        auto repairFlag = repair ? hoffman::RepairFlag::Repair : hoffman::RepairFlag::NoRepair;
        auto checkSigsFlag = checkSigs ? hoffman::CheckSigsFlag::CheckSigs : hoffman::CheckSigsFlag::NoCheckSigs;
        hoffman::copyStorePath(*srcStore->ptr, *dstStore->ptr, path->path, repairFlag, checkSigsFlag);
    }
    HOFFMANC_CATCH_ERRS
}

} // extern "C"
