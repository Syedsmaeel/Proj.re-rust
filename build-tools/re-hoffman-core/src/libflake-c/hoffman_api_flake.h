#ifndef HOFFMAN_API_FLAKE_H
#define HOFFMAN_API_FLAKE_H
/** @defgroup libflake libflake
 * @brief Bindings to the Hoffman Flakes library
 *
 * @{
 */
/** @file
 * @brief Main entry for the libflake C bindings
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
 * @brief A settings object for configuring the behavior of the hoffman-flake-c library.
 * @see hoffman_flake_settings_new
 * @see hoffman_flake_settings_free
 */
typedef struct hoffman_flake_settings hoffman_flake_settings;

/**
 * @brief Context and parameters for parsing a flake reference
 * @see hoffman_flake_reference_parse_flags_free
 * @see hoffman_flake_reference_parse_string
 */
typedef struct hoffman_flake_reference_parse_flags hoffman_flake_reference_parse_flags;

/**
 * @brief A reference to a flake
 *
 * A flake reference specifies how to fetch a flake.
 *
 * @see hoffman_flake_reference_from_string
 * @see hoffman_flake_reference_free
 */
typedef struct hoffman_flake_reference hoffman_flake_reference;

/**
 * @brief Parameters for locking a flake
 * @see hoffman_flake_lock_flags_new
 * @see hoffman_flake_lock_flags_free
 * @see hoffman_flake_lock
 */
typedef struct hoffman_flake_lock_flags hoffman_flake_lock_flags;

/**
 * @brief A flake with a suitable lock (file or otherwise)
 * @see hoffman_flake_lock
 * @see hoffman_locked_flake_free
 * @see hoffman_locked_flake_get_output_attrs
 */
typedef struct hoffman_locked_flake hoffman_locked_flake;

// Function prototypes
/**
 * Create a hoffman_flake_settings initialized with default values.
 * @param[out] context Optional, stores error information
 * @return A new hoffman_flake_settings or NULL on failure.
 * @see hoffman_flake_settings_free
 */
hoffman_flake_settings * hoffman_flake_settings_new(hoffman_c_context * context);

/**
 * @brief Release the resources associated with a hoffman_flake_settings.
 */
void hoffman_flake_settings_free(hoffman_flake_settings * settings);

/**
 * @brief Initialize a `hoffman_flake_settings` to contain `builtins.getFlake` and
 * potentially more.
 *
 * @warning This does not put the eval state in pure mode!
 *
 * @param[out] context Optional, stores error information
 * @param[in] settings The settings to use for e.g. `builtins.getFlake`
 * @param[in] builder The builder to modify
 */
hoffman_err hoffman_flake_settings_add_to_eval_state_builder(
    hoffman_c_context * context, hoffman_flake_settings * settings, hoffman_eval_state_builder * builder);

/**
 * @brief A new `hoffman_flake_reference_parse_flags` with defaults
 */
hoffman_flake_reference_parse_flags *
hoffman_flake_reference_parse_flags_new(hoffman_c_context * context, hoffman_flake_settings * settings);

/**
 * @brief Deallocate and release the resources associated with a `hoffman_flake_reference_parse_flags`.
 * Does not fail.
 * @param[in] flags the `hoffman_flake_reference_parse_flags *` to free
 */
void hoffman_flake_reference_parse_flags_free(hoffman_flake_reference_parse_flags * flags);

/**
 * @brief Provide a base directory for parsing relative flake references
 * @param[out] context Optional, stores error information
 * @param[in] flags The flags to modify
 * @param[in] baseDirectory The base directory to add
 * @param[in] baseDirectoryLen The length of baseDirectory
 * @return HOFFMAN_OK on success, HOFFMAN_ERR on failure
 */
hoffman_err hoffman_flake_reference_parse_flags_set_base_directory(
    hoffman_c_context * context,
    hoffman_flake_reference_parse_flags * flags,
    const char * baseDirectory,
    size_t baseDirectoryLen);

/**
 * @brief A new `hoffman_flake_lock_flags` with defaults
 * @param[in] settings Flake settings that may affect the defaults
 */
hoffman_flake_lock_flags * hoffman_flake_lock_flags_new(hoffman_c_context * context, hoffman_flake_settings * settings);

/**
 * @brief Deallocate and release the resources associated with a `hoffman_flake_lock_flags`.
 * Does not fail.
 * @param[in] settings the `hoffman_flake_lock_flags *` to free
 */
void hoffman_flake_lock_flags_free(hoffman_flake_lock_flags * settings);

/**
 * @brief Put the lock flags in a mode that checks whether the lock is up to date.
 * @param[out] context Optional, stores error information
 * @param[in] flags The flags to modify
 * @return HOFFMAN_OK on success, HOFFMAN_ERR on failure
 *
 * This causes `hoffman_flake_lock` to fail if the lock needs to be updated.
 */
hoffman_err hoffman_flake_lock_flags_set_mode_check(hoffman_c_context * context, hoffman_flake_lock_flags * flags);

/**
 * @brief Put the lock flags in a mode that updates the lock file in memory, if needed.
 * @param[out] context Optional, stores error information
 * @param[in] flags The flags to modify
 * @param[in] update Whether to allow updates
 *
 * This will cause `hoffman_flake_lock` to update the lock file in memory, if needed.
 */
hoffman_err hoffman_flake_lock_flags_set_mode_virtual(hoffman_c_context * context, hoffman_flake_lock_flags * flags);

/**
 * @brief Put the lock flags in a mode that updates the lock file on disk, if needed.
 * @param[out] context Optional, stores error information
 * @param[in] flags The flags to modify
 * @param[in] update Whether to allow updates
 *
 * This will cause `hoffman_flake_lock` to update the lock file on disk, if needed.
 */
hoffman_err hoffman_flake_lock_flags_set_mode_write_as_needed(hoffman_c_context * context, hoffman_flake_lock_flags * flags);

/**
 * @brief Add input overrides to the lock flags
 * @param[out] context Optional, stores error information
 * @param[in] flags The flags to modify
 * @param[in] inputPath The input path to override (must not be empty)
 * @param[in] flakeRef The flake reference to use as the override
 * @return HOFFMAN_ERR_HOFFMAN_ERROR if inputPath is empty
 *
 * This switches the `flags` to `hoffman_flake_lock_flags_set_mode_virtual` if not in mode
 * `hoffman_flake_lock_flags_set_mode_check`.
 */
hoffman_err hoffman_flake_lock_flags_add_input_override(
    hoffman_c_context * context, hoffman_flake_lock_flags * flags, const char * inputPath, hoffman_flake_reference * flakeRef);

/**
 * @brief Lock a flake, if not already locked.
 * @param[out] context Optional, stores error information
 * @param[in] settings The flake (and fetch) settings to use
 * @param[in] flags The locking flags to use
 * @param[in] flake The flake to lock
 */
hoffman_locked_flake * hoffman_flake_lock(
    hoffman_c_context * context,
    hoffman_fetchers_settings * fetchSettings,
    hoffman_flake_settings * settings,
    EvalState * eval_state,
    hoffman_flake_lock_flags * flags,
    hoffman_flake_reference * flake);

/**
 * @brief Deallocate and release the resources associated with a `hoffman_locked_flake`.
 * Does not fail.
 * @param[in] locked_flake the `hoffman_locked_flake *` to free
 */
void hoffman_locked_flake_free(hoffman_locked_flake * locked_flake);

/**
 * @brief Parse a URL-like string into a `hoffman_flake_reference`.
 *
 * @param[out] context **context** – Optional, stores error information
 * @param[in] fetchSettings **context** – The fetch settings to use
 * @param[in] flakeSettings **context** – The flake settings to use
 * @param[in] parseFlags **context** – Specific context and parameters such as base directory
 *
 * @param[in] str **input** – The URI-like string to parse
 * @param[in] strLen **input** – The length of `str`
 *
 * @param[out] flakeReferenceOut **result** – The resulting flake reference
 * @param[in] fragmentCallback **result** – A callback to call with the fragment part of the URL
 * @param[in] fragmentCallbackUserData **result** – User data to pass to the fragment callback
 *
 * @return HOFFMAN_OK on success, HOFFMAN_ERR on failure
 */
hoffman_err hoffman_flake_reference_and_fragment_from_string(
    hoffman_c_context * context,
    hoffman_fetchers_settings * fetchSettings,
    hoffman_flake_settings * flakeSettings,
    hoffman_flake_reference_parse_flags * parseFlags,
    const char * str,
    size_t strLen,
    hoffman_flake_reference ** flakeReferenceOut,
    hoffman_get_string_callback fragmentCallback,
    void * fragmentCallbackUserData);

/**
 * @brief Deallocate and release the resources associated with a `hoffman_flake_reference`.
 *
 * Does not fail.
 *
 * @param[in] store the `hoffman_flake_reference *` to free
 */
void hoffman_flake_reference_free(hoffman_flake_reference * store);

/**
 * @brief Get the output attributes of a flake.
 * @param[out] context Optional, stores error information
 * @param[in] settings The settings to use
 * @param[in] locked_flake the flake to get the output attributes from
 * @return A new hoffman_value or NULL on failure. Release the `hoffman_value` with `hoffman_value_decref`.
 */
hoffman_value * hoffman_locked_flake_get_output_attrs(
    hoffman_c_context * context, hoffman_flake_settings * settings, EvalState * evalState, hoffman_locked_flake * lockedFlake);

#ifdef __cplusplus
} // extern "C"
#endif

#endif
