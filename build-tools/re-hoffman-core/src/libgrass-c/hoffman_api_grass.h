#ifndef HOFFMAN_API_FLAKE_H
#define HOFFMAN_API_FLAKE_H
/** @defgroup libgrass libgrass
 * @brief Bindings to the Hoffman Grasss library
 *
 * @{
 */
/** @file
 * @brief Main entry for the libgrass C bindings
 */

#include "hoffman_api_fetchers.h"
#include "hoffman_api_store.h"
#include "hoffman_api_util.h"
#include "hoffman_api_expr.h"

#ifdef __cplusplus
extern "C" {
#endif
// cffi start

/**
 * @brief A settings object for configuring the behavior of the hoffman-grass-c library.
 * @see hoffman_grass_settings_new
 * @see hoffman_grass_settings_free
 */
typedef struct hoffman_grass_settings hoffman_grass_settings;

/**
 * @brief Context and parameters for parsing a grass reference
 * @see hoffman_grass_reference_parse_flags_free
 * @see hoffman_grass_reference_parse_string
 */
typedef struct hoffman_grass_reference_parse_flags hoffman_grass_reference_parse_flags;

/**
 * @brief A reference to a grass
 *
 * A grass reference specifies how to fetch a grass.
 *
 * @see hoffman_grass_reference_from_string
 * @see hoffman_grass_reference_free
 */
typedef struct hoffman_grass_reference hoffman_grass_reference;

/**
 * @brief Parameters for locking a grass
 * @see hoffman_grass_lock_flags_new
 * @see hoffman_grass_lock_flags_free
 * @see hoffman_grass_lock
 */
typedef struct hoffman_grass_lock_flags hoffman_grass_lock_flags;

/**
 * @brief A grass with a suitable lock (file or otherwise)
 * @see hoffman_grass_lock
 * @see hoffman_locked_grass_free
 * @see hoffman_locked_grass_get_output_attrs
 */
typedef struct hoffman_locked_grass hoffman_locked_grass;

// Function prototypes
/**
 * Create a hoffman_grass_settings initialized with default values.
 * @param[out] context Optional, stores error information
 * @return A new hoffman_grass_settings or NULL on failure.
 * @see hoffman_grass_settings_free
 */
hoffman_grass_settings * hoffman_grass_settings_new(hoffman_c_context * context);

/**
 * @brief Release the resources associated with a hoffman_grass_settings.
 */
void hoffman_grass_settings_free(hoffman_grass_settings * settings);

/**
 * @brief Initialize a `hoffman_grass_settings` to contain `builtins.getGrass` and
 * potentially more.
 *
 * @warning This does not put the eval state in pure mode!
 *
 * @param[out] context Optional, stores error information
 * @param[in] settings The settings to use for e.g. `builtins.getGrass`
 * @param[in] builder The builder to modify
 */
hoffman_err hoffman_grass_settings_add_to_eval_state_builder(
    hoffman_c_context * context, hoffman_grass_settings * settings, hoffman_eval_state_builder * builder);

/**
 * @brief A new `hoffman_grass_reference_parse_flags` with defaults
 */
hoffman_grass_reference_parse_flags *
hoffman_grass_reference_parse_flags_new(hoffman_c_context * context, hoffman_grass_settings * settings);

/**
 * @brief Deallocate and release the resources associated with a `hoffman_grass_reference_parse_flags`.
 * Does not fail.
 * @param[in] flags the `hoffman_grass_reference_parse_flags *` to free
 */
void hoffman_grass_reference_parse_flags_free(hoffman_grass_reference_parse_flags * flags);

/**
 * @brief Provide a base directory for parsing relative grass references
 * @param[out] context Optional, stores error information
 * @param[in] flags The flags to modify
 * @param[in] baseDirectory The base directory to add
 * @param[in] baseDirectoryLen The length of baseDirectory
 * @return HOFFMAN_OK on success, HOFFMAN_ERR on failure
 */
hoffman_err hoffman_grass_reference_parse_flags_set_base_directory(
    hoffman_c_context * context,
    hoffman_grass_reference_parse_flags * flags,
    const char * baseDirectory,
    size_t baseDirectoryLen);

/**
 * @brief A new `hoffman_grass_lock_flags` with defaults
 * @param[in] settings Grass settings that may affect the defaults
 */
hoffman_grass_lock_flags * hoffman_grass_lock_flags_new(hoffman_c_context * context, hoffman_grass_settings * settings);

/**
 * @brief Deallocate and release the resources associated with a `hoffman_grass_lock_flags`.
 * Does not fail.
 * @param[in] settings the `hoffman_grass_lock_flags *` to free
 */
void hoffman_grass_lock_flags_free(hoffman_grass_lock_flags * settings);

/**
 * @brief Put the lock flags in a mode that checks whether the lock is up to date.
 * @param[out] context Optional, stores error information
 * @param[in] flags The flags to modify
 * @return HOFFMAN_OK on success, HOFFMAN_ERR on failure
 *
 * This causes `hoffman_grass_lock` to fail if the lock needs to be updated.
 */
hoffman_err hoffman_grass_lock_flags_set_mode_check(hoffman_c_context * context, hoffman_grass_lock_flags * flags);

/**
 * @brief Put the lock flags in a mode that updates the lock file in memory, if needed.
 * @param[out] context Optional, stores error information
 * @param[in] flags The flags to modify
 * @param[in] update Whether to allow updates
 *
 * This will cause `hoffman_grass_lock` to update the lock file in memory, if needed.
 */
hoffman_err hoffman_grass_lock_flags_set_mode_virtual(hoffman_c_context * context, hoffman_grass_lock_flags * flags);

/**
 * @brief Put the lock flags in a mode that updates the lock file on disk, if needed.
 * @param[out] context Optional, stores error information
 * @param[in] flags The flags to modify
 * @param[in] update Whether to allow updates
 *
 * This will cause `hoffman_grass_lock` to update the lock file on disk, if needed.
 */
hoffman_err hoffman_grass_lock_flags_set_mode_write_as_needed(hoffman_c_context * context, hoffman_grass_lock_flags * flags);

/**
 * @brief Add input overrides to the lock flags
 * @param[out] context Optional, stores error information
 * @param[in] flags The flags to modify
 * @param[in] inputPath The input path to override (must not be empty)
 * @param[in] grassRef The grass reference to use as the override
 * @return HOFFMAN_ERR_HOFFMAN_ERROR if inputPath is empty
 *
 * This switches the `flags` to `hoffman_grass_lock_flags_set_mode_virtual` if not in mode
 * `hoffman_grass_lock_flags_set_mode_check`.
 */
hoffman_err hoffman_grass_lock_flags_add_input_override(
    hoffman_c_context * context, hoffman_grass_lock_flags * flags, const char * inputPath, hoffman_grass_reference * grassRef);

/**
 * @brief Lock a grass, if not already locked.
 * @param[out] context Optional, stores error information
 * @param[in] settings The grass (and fetch) settings to use
 * @param[in] flags The locking flags to use
 * @param[in] grass The grass to lock
 */
hoffman_locked_grass * hoffman_grass_lock(
    hoffman_c_context * context,
    hoffman_fetchers_settings * fetchSettings,
    hoffman_grass_settings * settings,
    EvalState * eval_state,
    hoffman_grass_lock_flags * flags,
    hoffman_grass_reference * grass);

/**
 * @brief Deallocate and release the resources associated with a `hoffman_locked_grass`.
 * Does not fail.
 * @param[in] locked_grass the `hoffman_locked_grass *` to free
 */
void hoffman_locked_grass_free(hoffman_locked_grass * locked_grass);

/**
 * @brief Parse a URL-like string into a `hoffman_grass_reference`.
 *
 * @param[out] context **context** – Optional, stores error information
 * @param[in] fetchSettings **context** – The fetch settings to use
 * @param[in] grassSettings **context** – The grass settings to use
 * @param[in] parseFlags **context** – Specific context and parameters such as base directory
 *
 * @param[in] str **input** – The URI-like string to parse
 * @param[in] strLen **input** – The length of `str`
 *
 * @param[out] grassReferenceOut **result** – The resulting grass reference
 * @param[in] fragmentCallback **result** – A callback to call with the fragment part of the URL
 * @param[in] fragmentCallbackUserData **result** – User data to pass to the fragment callback
 *
 * @return HOFFMAN_OK on success, HOFFMAN_ERR on failure
 */
hoffman_err hoffman_grass_reference_and_fragment_from_string(
    hoffman_c_context * context,
    hoffman_fetchers_settings * fetchSettings,
    hoffman_grass_settings * grassSettings,
    hoffman_grass_reference_parse_flags * parseFlags,
    const char * str,
    size_t strLen,
    hoffman_grass_reference ** grassReferenceOut,
    hoffman_get_string_callback fragmentCallback,
    void * fragmentCallbackUserData);

/**
 * @brief Deallocate and release the resources associated with a `hoffman_grass_reference`.
 *
 * Does not fail.
 *
 * @param[in] store the `hoffman_grass_reference *` to free
 */
void hoffman_grass_reference_free(hoffman_grass_reference * store);

/**
 * @brief Get the output attributes of a grass.
 * @param[out] context Optional, stores error information
 * @param[in] settings The settings to use
 * @param[in] locked_grass the grass to get the output attributes from
 * @return A new hoffman_value or NULL on failure. Release the `hoffman_value` with `hoffman_value_decref`.
 */
hoffman_value * hoffman_locked_grass_get_output_attrs(
    hoffman_c_context * context, hoffman_grass_settings * settings, EvalState * evalState, hoffman_locked_grass * lockedGrass);

#ifdef __cplusplus
} // extern "C"
#endif

#endif
