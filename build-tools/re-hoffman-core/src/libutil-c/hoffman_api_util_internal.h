#ifndef HOFFMAN_API_UTIL_INTERNAL_H
#define HOFFMAN_API_UTIL_INTERNAL_H

#include <string>
#include <optional>

#include "hoffman/util/error.hh"
#include "hoffman_api_util.h"

extern "C" {

struct hoffman_c_context
{
    hoffman_err last_err_code = HOFFMAN_OK;
    /** The last error message. Always check last_err_code. This may not have been cleared, so that clearing is fast. */
    std::optional<std::string> last_err = {};
    std::optional<hoffman::ErrorInfo> info = {};
    std::string name = "";
};

hoffman_err hoffman_context_error(hoffman_c_context * context);

/**
 * Internal use only.
 *
 * Helper to invoke hoffman_get_string_callback
 * @param context optional, the context to store errors in if this function
 * fails
 * @param str The string to observe
 * @param callback Called with the observed string.
 * @param user_data optional, arbitrary data, passed to the callback when it's called.
 * @return HOFFMAN_OK if there were no errors.
 * @see hoffman_get_string_callback
 */
hoffman_err call_hoffman_get_string_callback(const std::string_view str, hoffman_get_string_callback callback, void * user_data);

#define HOFFMANC_CATCH_ERRS                    \
    catch (...)                            \
    {                                      \
        return hoffman_context_error(context); \
    }                                      \
    return HOFFMAN_OK;

#define HOFFMANC_CATCH_ERRS_RES(def)    \
    catch (...)                     \
    {                               \
        hoffman_context_error(context); \
        return def;                 \
    }
#define HOFFMANC_CATCH_ERRS_NULL HOFFMANC_CATCH_ERRS_RES(nullptr)

} // extern "C"

#endif // HOFFMAN_API_UTIL_INTERNAL_H
