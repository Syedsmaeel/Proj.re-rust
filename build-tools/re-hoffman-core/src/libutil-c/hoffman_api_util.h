#ifndef HOFFMAN_API_UTIL_H
#define HOFFMAN_API_UTIL_H
/**
 * @defgroup libutil libutil
 * @brief C bindings for hoffman libutil
 *
 * libutil is used for functionality shared between
 * different Hoffman modules.
 * @{
 */
/** @file
 * @brief Main entry for the libutil C bindings
 *
 * Also contains error handling utilities
 */

#ifdef __cplusplus
extern "C" {
#endif
// cffi start

/** @defgroup errors Handling errors
 * @brief Dealing with errors from the Hoffman side
 *
 * To handle errors that can be returned from the Hoffman API,
 * a hoffman_c_context can be passed to any function that potentially returns an
 * error.
 *
 * Error information will be stored in this context, and can be retrieved
 * using hoffman_err_code and hoffman_err_msg.
 *
 * Passing NULL instead will cause the API to throw C++ errors.
 *
 * Example:
 * @code{.c}
 * int main() {
 *     hoffman_c_context* ctx = hoffman_c_context_create();
 *     hoffman_libutil_init(ctx);
 *     if (hoffman_err_code(ctx) != HOFFMAN_OK) {
 *         printf("error: %s\n", hoffman_err_msg(NULL, ctx, NULL));
 *         return 1;
 *     }
 *     return 0;
 * }
 * @endcode
 *  @{
 */
// Error codes
/**
 * @brief Type for error codes in the Hoffman system
 *
 * This type can have one of several predefined constants:
 * - HOFFMAN_OK: No error occurred (0)
 * - HOFFMAN_ERR_UNKNOWN: An unknown error occurred (-1)
 * - HOFFMAN_ERR_OVERFLOW: An overflow error occurred (-2)
 * - HOFFMAN_ERR_KEY: A key/index access error occurred in C API functions (-3)
 * - HOFFMAN_ERR_HOFFMAN_ERROR: A generic Hoffman error occurred (-4)
 */
enum hoffman_err {

    /**
     * @brief No error occurred.
     *
     * This error code is returned when no error has occurred during the function
     * execution.
     */
    HOFFMAN_OK = 0,

    /**
     * @brief An unknown error occurred.
     *
     * This error code is returned when an unknown error occurred during the
     * function execution.
     */
    HOFFMAN_ERR_UNKNOWN = -1,

    /**
     * @brief An overflow error occurred.
     *
     * This error code is returned when an overflow error occurred during the
     * function execution.
     */
    HOFFMAN_ERR_OVERFLOW = -2,

    /**
     * @brief A key/index access error occurred in C API functions.
     *
     * This error code is returned when accessing a key, index, or identifier that
     * does not exist in C API functions. Common scenarios include:
     * - Setting keys that don't exist (hoffman_setting_get, hoffman_setting_set)
     * - List indices that are out of bounds (hoffman_get_list_byidx*)
     * - Attribute names that don't exist (hoffman_get_attr_byname*)
     * - Attribute indices that are out of bounds (hoffman_get_attr_byidx*, hoffman_get_attr_name_byidx)
     *
     * This error typically indicates incorrect usage or assumptions about data structure
     * contents, rather than internal Hoffman evaluation errors.
     *
     * @note This error code should ONLY be returned by C API functions themselves,
     * not by underlying Hoffman evaluation. For example, evaluating `{}.foo` in Hoffman
     * will throw a normal error (HOFFMAN_ERR_HOFFMAN_ERROR), not HOFFMAN_ERR_KEY.
     */
    HOFFMAN_ERR_KEY = -3,

    /**
     * @brief A generic Hoffman error occurred.
     *
     * This error code is returned when a generic Hoffman error occurred during the
     * function execution.
     */
    HOFFMAN_ERR_HOFFMAN_ERROR = -4,

    /**
     * @brief A recoverable error occurred.
     *
     * This is used primarily by C API *consumers* to communicate that a failed
     * primop call should be retried on the next evaluation attempt.
     */
    HOFFMAN_ERR_RECOVERABLE = -5,
};

typedef enum hoffman_err hoffman_err;

/**
 * @brief Verbosity level
 *
 * @note This should be kept in sync with the C++ implementation (hoffman::Verbosity)
 */
enum hoffman_verbosity {
    HOFFMAN_LVL_ERROR = 0,
    HOFFMAN_LVL_WARN,
    HOFFMAN_LVL_NOTICE,
    HOFFMAN_LVL_INFO,
    HOFFMAN_LVL_TALKATIVE,
    HOFFMAN_LVL_CHATTY,
    HOFFMAN_LVL_DEBUG,
    HOFFMAN_LVL_VOMIT,
};

typedef enum hoffman_verbosity hoffman_verbosity;

/**
 * @brief This object stores error state.
 * @struct hoffman_c_context
 *
 * Passed as a first parameter to functions that can fail, to store error
 * information.
 *
 * Optional wherever it can be used, passing NULL instead will throw a C++
 * exception.
 *
 * The struct is laid out so that it can also be cast to hoffman_err* to inspect
 * directly:
 * @code{.c}
 * assert(*(hoffman_err*)ctx == HOFFMAN_OK);
 * @endcode
 * @note These can be reused between different function calls,
 *  but make sure not to use them for multiple calls simultaneously (which can
 * happen in callbacks).
 */
typedef struct hoffman_c_context hoffman_c_context;

/**
 * @brief Called to get the value of a string owned by Hoffman.
 *
 * The `start` data is borrowed and the function must not assume that the buffer persists after it returns.
 * @warning Don't assume that the string is NUL-terminated.
 *
 * @param[in] start the string to copy.
 * @param[in] n the string length.
 * @param[in] user_data optional, arbitrary data, passed to the hoffman_get_string_callback when it's called.
 */
typedef void (*hoffman_get_string_callback)(const char * start, unsigned int n, void * user_data);

// Function prototypes

/**
 * @brief Allocate a new hoffman_c_context.
 * @throws std::bad_alloc
 * @return allocated hoffman_c_context, owned by the caller. Free using
 * `hoffman_c_context_free`.
 */
hoffman_c_context * hoffman_c_context_create();
/**
 * @brief Free a hoffman_c_context. Does not fail.
 * @param[out] context The context to free, mandatory.
 */
void hoffman_c_context_free(hoffman_c_context * context);
/**
 *  @}
 */

/**
 * @brief Initializes hoffman_libutil and its dependencies.
 *
 * This function can be called multiple times, but should be called at least
 * once prior to any other hoffman function.
 *
 * @param[out] context Optional, stores error information
 * @return HOFFMAN_OK if the initialization is successful, or an error code
 * otherwise.
 */
hoffman_err hoffman_libutil_init(hoffman_c_context * context);

/** @defgroup settings Hoffman configuration settings
 *  @{
 */
/**
 * @brief Retrieves a setting from the hoffman global configuration.
 *
 * This function requires hoffman_libutil_init() to be called at least once prior to
 * its use.
 *
 * @param[out] context optional, Stores error information
 * @param[in] key The key of the setting to retrieve.
 * @param[in] callback Called with the setting value.
 * @param[in] user_data optional, arbitrary data, passed to the callback when it's called.
 * @see hoffman_get_string_callback
 * @return HOFFMAN_ERR_KEY if the setting is unknown, or HOFFMAN_OK if the setting was retrieved
 * successfully.
 */
hoffman_err hoffman_setting_get(hoffman_c_context * context, const char * key, hoffman_get_string_callback callback, void * user_data);

/**
 * @brief Sets a setting in the hoffman global configuration.
 *
 * Use "extra-<setting name>" to append to the setting's value.
 *
 * Settings only apply for new State%s. Call hoffman_plugins_init() when you are
 * done with the settings to load any plugins.
 *
 * @param[out] context optional, Stores error information
 * @param[in] key The key of the setting to set.
 * @param[in] value The value to set for the setting.
 * @return HOFFMAN_ERR_KEY if the setting is unknown, or HOFFMAN_OK if the setting was
 * set successfully.
 */
hoffman_err hoffman_setting_set(hoffman_c_context * context, const char * key, const char * value);

/**
 *  @}
 */
// todo: hoffman_plugins_init()

/**
 * @brief Retrieves the hoffman library version.
 *
 * Does not fail.
 * @return A static string representing the version of the hoffman library.
 */
const char * hoffman_version_get();

/** @addtogroup errors
 *  @{
 */
/**
 * @brief Retrieves the most recent error message from a context.
 *
 * @pre This function should only be called after a previous hoffman function has
 * returned an error.
 *
 * @param[out] context optional, the context to store errors in if this function
 * fails
 * @param[in] ctx the context to retrieve the error message from
 * @param[out] n optional: a pointer to an unsigned int that is set to the
 * length of the error.
 * @return nullptr if no error message was ever set,
 *         a borrowed pointer to the error message otherwise, which is valid
 *         until the next call to a Hoffman function, or until the context is
 *         destroyed.
 */
const char * hoffman_err_msg(hoffman_c_context * context, const hoffman_c_context * ctx, unsigned int * n);

/**
 * @brief Retrieves the error message from errorInfo in a context.
 *
 * Used to inspect hoffman Error messages.
 *
 * @pre This function should only be called after a previous hoffman function has
 * returned a HOFFMAN_ERR_HOFFMAN_ERROR
 *
 * @param[out] context optional, the context to store errors in if this function
 * fails
 * @param[in] read_context the context to retrieve the error message from.
 * @param[in] callback Called with the error message.
 * @param[in] user_data optional, arbitrary data, passed to the callback when it's called.
 * @see hoffman_get_string_callback
 * @return HOFFMAN_OK if there were no errors, an error code otherwise.
 */
hoffman_err hoffman_err_info_msg(
    hoffman_c_context * context, const hoffman_c_context * read_context, hoffman_get_string_callback callback, void * user_data);

/**
 * @brief Retrieves the error name from a context.
 *
 * Used to inspect hoffman Error messages.
 *
 * @pre This function should only be called after a previous hoffman function has
 * returned a HOFFMAN_ERR_HOFFMAN_ERROR
 *
 * @param context optional, the context to store errors in if this function
 * fails
 * @param[in] read_context the context to retrieve the error message from
 * @param[in] callback Called with the error name.
 * @param[in] user_data optional, arbitrary data, passed to the callback when it's called.
 * @see hoffman_get_string_callback
 * @return HOFFMAN_OK if there were no errors, an error code otherwise.
 */
hoffman_err hoffman_err_name(
    hoffman_c_context * context, const hoffman_c_context * read_context, hoffman_get_string_callback callback, void * user_data);

/**
 * @brief Retrieves the most recent error code from a hoffman_c_context
 *
 * Equivalent to reading the first field of the context.
 *
 * Does not fail
 *
 * @param[in] read_context the context to retrieve the error message from
 * @return most recent error code stored in the context.
 */
hoffman_err hoffman_err_code(const hoffman_c_context * read_context);

/**
 * @brief Set an error message on a hoffman context.
 *
 * This should be used when you want to throw an error from a PrimOp callback.
 *
 * All other use is internal to the API.
 *
 * @param context context to write the error message to, required unless C++ exceptions are supported
 * @param err The error code to set and return
 * @param msg The error message to set. This string is copied.
 * @returns the error code set
 */
hoffman_err hoffman_set_err_msg(hoffman_c_context * context, hoffman_err err, const char * msg);

/**
 * @brief Clear the error message from a hoffman context.
 *
 * This is performed implicitly by all functions that accept a context, so
 * this won't be necessary in most cases.
 * However, if you want to clear the error message without calling another
 * function, you can use this.
 *
 * Example use case: a higher order function that helps with error handling,
 * to make it more robust in the following scenario:
 *
 * 1. A previous call failed, and the error was caught and handled.
 * 2. The context is reused with our error handling helper function.
 * 3. The callback passed to the helper function doesn't actually make a call to
 *    a Hoffman function.
 * 4. The handled error is raised again, from an unrelated call.
 *
 * This failure can be avoided by clearing the error message after handling it.
 */
void hoffman_clear_err(hoffman_c_context * context);

/**
 * @brief Sets the verbosity level
 *
 * @param[out] context Optional, additional error context.
 * @param[in] level Verbosity level
 */
hoffman_err hoffman_set_verbosity(hoffman_c_context * context, hoffman_verbosity level);

/**
 *  @}
 */

// cffi end
#ifdef __cplusplus
}
#endif

/** @} */
#endif // HOFFMAN_API_UTIL_H
