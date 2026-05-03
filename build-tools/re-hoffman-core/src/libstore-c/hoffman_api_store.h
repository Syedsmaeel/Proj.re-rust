#ifndef HOFFMAN_API_STORE_H
#define HOFFMAN_API_STORE_H
/**
 * @defgroup libstore libstore
 * @brief C bindings for hoffman libstore
 *
 * libstore is used for talking to a Hoffman store
 * @{
 */
/** @file
 * @brief Main entry for the libstore C bindings
 */

#include "hoffman_api_util.h"
#include "hoffman_api_store/store_path.h"
#include "hoffman_api_store/derivation.h"
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif
// cffi start

/** @brief Reference to a Hoffman store */
typedef struct Store Store;

/**
 * @brief Initializes the Hoffman store library
 *
 * This function should be called before creating a Store
 * This function can be called multiple times.
 *
 * @param[out] context Optional, stores error information
 * @return HOFFMAN_OK if the initialization was successful, an error code otherwise.
 */
hoffman_err hoffman_libstore_init(hoffman_c_context * context);

/**
 * @brief Like hoffman_libstore_init, but does not load the Hoffman configuration.
 *
 * This is useful when external configuration is not desired, such as when running unit tests.
 */
hoffman_err hoffman_libstore_init_no_load_config(hoffman_c_context * context);

/**
 * @brief Open a hoffman store.
 *
 * Store instances may share state and resources behind the scenes.
 *
 * @param[out] context Optional, stores error information
 *
 * @param[in] uri @parblock
 *   URI of the Hoffman store, copied.
 *
 *   If `NULL`, the store from the settings will be used.
 *   Note that `"auto"` holds a strange middle ground, reading part of the general environment, but not all of it. It
 * ignores `HOFFMAN_REMOTE` and the `store` option. For this reason, `NULL` is most likely the better choice.
 *
 *   For supported store URLs, see [*Store URL format* in the Hoffman Reference
 * Manual](https://hoffman.dev/manual/hoffman/stable/store/types/#store-url-format).
 * @endparblock
 *
 * @param[in] params @parblock
 *   optional, null-terminated array of key-value pairs, e.g. {{"endpoint",
 * "https://s3.local"}}.
 *
 *   See [*Store Types* in the Hoffman Reference Manual](https://hoffman.dev/manual/hoffman/stable/store/types).
 * @endparblock
 *
 * @return a Store pointer, NULL in case of errors
 *
 * @see hoffman_store_free
 */
Store * hoffman_store_open(hoffman_c_context * context, const char * uri, const char *** params);

/**
 * @brief Deallocate a hoffman store and free any resources if not also held by other Store instances.
 *
 * Does not fail.
 *
 * @param[in] store the store to free
 */
void hoffman_store_free(Store * store);

/**
 * @brief get the URI of a hoffman store
 * @param[out] context Optional, stores error information
 * @param[in] store hoffman store reference
 * @param[in] callback Called with the URI.
 * @param[in] user_data optional, arbitrary data, passed to the callback when it's called.
 * @see hoffman_get_string_callback
 * @return error code, HOFFMAN_OK on success.
 */
hoffman_err hoffman_store_get_uri(hoffman_c_context * context, Store * store, hoffman_get_string_callback callback, void * user_data);

/**
 * @brief get the storeDir of a Hoffman store, typically `"/hoffman/store"`
 * @param[out] context Optional, stores error information
 * @param[in] store hoffman store reference
 * @param[in] callback Called with the URI.
 * @param[in] user_data optional, arbitrary data, passed to the callback when it's called.
 * @see hoffman_get_string_callback
 * @return error code, HOFFMAN_OK on success.
 */
hoffman_err
hoffman_store_get_storedir(hoffman_c_context * context, Store * store, hoffman_get_string_callback callback, void * user_data);

/**
 * @brief Parse a Hoffman store path that includes the store dir into a StorePath
 *
 * @note Don't forget to free this path using hoffman_store_path_free()!
 * @param[out] context Optional, stores error information
 * @param[in] store hoffman store reference
 * @param[in] path Path string to parse, copied
 * @return owned store path, NULL on error
 */
StorePath * hoffman_store_parse_path(hoffman_c_context * context, Store * store, const char * path);

/**
 * @brief Check if a StorePath is valid (i.e. that corresponding store object and its closure of references exists in
 * the store)
 * @param[out] context Optional, stores error information
 * @param[in] store Hoffman Store reference
 * @param[in] path Path to check
 * @return true or false, error info in context
 */
bool hoffman_store_is_valid_path(hoffman_c_context * context, Store * store, const StorePath * path);

/**
 * @brief Get the physical location of a store path
 *
 * A store may reside at a different location than its `storeDir` suggests.
 * This situation is called a relocated store.
 * Relocated stores are used during HoffmanOS installation, as well as in restricted computing environments that don't offer
 * a writable `/hoffman/store`.
 *
 * Not all types of stores support this operation.
 *
 * @param[in] context Optional, stores error information
 * @param[in] store hoffman store reference
 * @param[in] path the path to get the real path from
 * @param[in] callback called with the real path
 * @param[in] user_data arbitrary data, passed to the callback when it's called.
 */
hoffman_err hoffman_store_real_path(
    hoffman_c_context * context, Store * store, StorePath * path, hoffman_get_string_callback callback, void * user_data);

// hoffman_err hoffman_store_ensure(Store*, const char*);
// hoffman_err hoffman_store_build_paths(Store*);
/**
 * @brief Realise a Hoffman store path
 *
 * Blocking, calls callback once for each realised output.
 *
 * @note When working with expressions, consider using e.g. hoffman_string_realise to get the output. `.drvPath` may not be
 * accurate or available in the future. See https://github.com/HoffmanOS/hoffman/issues/6507
 *
 * @param[out] context Optional, stores error information
 * @param[in] store Hoffman Store reference
 * @param[in] path Path to build
 * @param[in] userdata data to pass to every callback invocation
 * @param[in] callback called for every realised output
 * @return HOFFMAN_OK if the build succeeded, or an error code if the build/scheduling/outputs/copying/etc failed.
 *         On error, the callback is never invoked and error information is stored in context.
 */
hoffman_err hoffman_store_realise(
    hoffman_c_context * context,
    Store * store,
    StorePath * path,
    void * userdata,
    void (*callback)(void * userdata, const char * outname, const StorePath * out));

/**
 * @brief get the version of a hoffman store.
 *
 * If the store doesn't have a version (like the dummy store), returns an empty string.
 *
 * @param[out] context Optional, stores error information
 * @param[in] store hoffman store reference
 * @param[in] callback Called with the version.
 * @param[in] user_data optional, arbitrary data, passed to the callback when it's called.
 * @see hoffman_get_string_callback
 * @return error code, HOFFMAN_OK on success.
 */
hoffman_err
hoffman_store_get_version(hoffman_c_context * context, Store * store, hoffman_get_string_callback callback, void * user_data);

/**
 * @brief Create a `hoffman_derivation` from a JSON representation of that derivation.
 *
 * @note Unlike `hoffman_derivation_to_json`, this needs a `Store`. This is because
 * over time we expect the internal representation of derivations in Hoffman to
 * differ from accepted derivation formats. The store argument is here to help
 * any logic needed to convert from JSON to the internal representation, in
 * excess of just parsing.
 *
 * @param[out] context Optional, stores error information.
 * @param[in] store hoffman store reference.
 * @param[in] json JSON of the derivation as a string.
 * @return A new derivation, or NULL on error. Free with `hoffman_derivation_free` when done using the `hoffman_derivation`.
 */
hoffman_derivation * hoffman_derivation_from_json(hoffman_c_context * context, Store * store, const char * json);

/**
 * @brief Add the given `hoffman_derivation` to the given store
 *
 * @param[out] context Optional, stores error information.
 * @param[in] store hoffman store reference. The derivation will be inserted here.
 * @param[in] derivation hoffman_derivation to insert into the given store.
 */
StorePath * hoffman_add_derivation(hoffman_c_context * context, Store * store, hoffman_derivation * derivation);

/**
 * @brief Copy the closure of `path` from `srcStore` to `dstStore`.
 *
 * @param[out] context Optional, stores error information
 * @param[in] srcStore hoffman source store reference
 * @param[in] dstStore hoffman destination store reference
 * @param[in] path Path to copy
 */
hoffman_err hoffman_store_copy_closure(hoffman_c_context * context, Store * srcStore, Store * dstStore, StorePath * path);

/**
 * @brief Gets the closure of a specific store path
 *
 * @note The callback borrows each StorePath only for the duration of the call.
 *
 * @param[out] context Optional, stores error information
 * @param[in] store hoffman store reference
 * @param[in] store_path The path to compute from
 * @param[in] flip_direction If false, compute the forward closure (paths referenced by any store path in the closure).
 *                           If true, compute the backward closure (paths that reference any store path in the closure).
 * @param[in] include_outputs If flip_direction is false: for any derivation in the closure, include its outputs.
 *                            If flip_direction is true: for any output in the closure, include derivations that produce
 *                            it.
 * @param[in] include_derivers If flip_direction is false: for any output in the closure, include the derivation that
 *                             produced it.
 *                             If flip_direction is true: for any derivation in the closure, include its outputs.
 * @param[in] callback The function to call for every store path, in no particular order
 * @param[in] userdata The userdata to pass to the callback
 */
hoffman_err hoffman_store_get_fs_closure(
    hoffman_c_context * context,
    Store * store,
    const StorePath * store_path,
    bool flip_direction,
    bool include_outputs,
    bool include_derivers,
    void * userdata,
    void (*callback)(hoffman_c_context * context, void * userdata, const StorePath * store_path));

/**
 * @brief Returns the derivation associated with the store path
 *
 * @param[out] context Optional, stores error information
 * @param[in] store The hoffman store
 * @param[in] path The hoffman store path
 * @return A new derivation, or NULL on error. Free with `hoffman_derivation_free` when done using the `hoffman_derivation`.
 */
hoffman_derivation * hoffman_store_drv_from_store_path(hoffman_c_context * context, Store * store, const StorePath * path);

/**
 * @brief Query the full store path given the hash part of a valid store
 * path, or empty if no matching path is found.
 *
 * @param[out] context Optional, stores error information
 * @param[in] store hoffman store reference
 * @param[in] hash Hash part of path as a string
 * @return Store path reference, NULL if no matching path is found.
 */
StorePath * hoffman_store_query_path_from_hash_part(hoffman_c_context * context, Store * store, const char * hash);

/**
 * @brief Copy a path from one store to another.
 *
 * @param[out] context Optional, stores error information
 * @param[in] srcStore hoffman source store reference
 * @param[in] dstStore hoffman destination store reference
 * @param[in] path The path to copy
 * @param[in] repair Whether to repair the path
 * @param[in] checkSigs Whether to check path signatures are trusted before copying
 */
hoffman_err hoffman_store_copy_path(
    hoffman_c_context * context, Store * srcStore, Store * dstStore, const StorePath * path, bool repair, bool checkSigs);

// cffi end
#ifdef __cplusplus
}
#endif
/**
 * @}
 */
#endif // HOFFMAN_API_STORE_H
