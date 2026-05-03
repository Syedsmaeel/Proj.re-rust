#include "hoffman_api_util.h"
#include "hoffman/util/config-global.hh"
#include "hoffman/util/error.hh"
#include "hoffman_api_util_internal.h"
#include "hoffman/util/util.hh"

#include <cxxabi.h>
#include <typeinfo>

#include "hoffman_api_util_config.h"

extern "C" {

hoffman_c_context * hoffman_c_context_create()
{
    try {
        return new hoffman_c_context();
    } catch (...) {
        return nullptr;
    }
}

void hoffman_c_context_free(hoffman_c_context * context)
{
    delete context;
}

hoffman_err hoffman_context_error(hoffman_c_context * context)
{
    if (context == nullptr) {
        throw;
    }
    try {
        throw;
    } catch (hoffman::Error & e) {
        /* Storing this exception is annoying, take what we need here */
        context->last_err = e.what();
        context->info = e.info();
        int status;
        const char * demangled = abi::__cxa_demangle(typeid(e).name(), 0, 0, &status);
        if (demangled) {
            context->name = demangled;
            free((void *) demangled);
        } else {
            context->name = typeid(e).name();
        }
        context->last_err_code = HOFFMAN_ERR_HOFFMAN_ERROR;
        return context->last_err_code;
    } catch (const std::exception & e) {
        context->last_err = e.what();
        context->last_err_code = HOFFMAN_ERR_UNKNOWN;
        return context->last_err_code;
    }
    // unreachable
}

hoffman_err hoffman_set_err_msg(hoffman_c_context * context, hoffman_err err, const char * msg)
{
    if (context == nullptr) {
        // todo last_err_code
        throw hoffman::Error("Hoffman C api error: %s", msg);
    }
    context->last_err_code = err;
    context->last_err = msg;
    return err;
}

void hoffman_clear_err(hoffman_c_context * context)
{
    if (context)
        context->last_err_code = HOFFMAN_OK;
}

const char * hoffman_version_get()
{
    return PACKAGE_VERSION;
}

// Implementations

hoffman_err hoffman_setting_get(hoffman_c_context * context, const char * key, hoffman_get_string_callback callback, void * user_data)
{
    if (context)
        context->last_err_code = HOFFMAN_OK;
    try {
        std::map<std::string, hoffman::AbstractConfig::SettingInfo> settings;
        hoffman::globalConfig.getSettings(settings);
        if (settings.contains(key)) {
            return call_hoffman_get_string_callback(settings[key].value, callback, user_data);
        } else {
            return hoffman_set_err_msg(context, HOFFMAN_ERR_KEY, "Setting not found");
        }
    }
    HOFFMANC_CATCH_ERRS
}

hoffman_err hoffman_setting_set(hoffman_c_context * context, const char * key, const char * value)
{
    if (context)
        context->last_err_code = HOFFMAN_OK;
    if (hoffman::globalConfig.set(key, value))
        return HOFFMAN_OK;
    else {
        return hoffman_set_err_msg(context, HOFFMAN_ERR_KEY, "Setting not found");
    }
}

hoffman_err hoffman_libutil_init(hoffman_c_context * context)
{
    if (context)
        context->last_err_code = HOFFMAN_OK;
    try {
        hoffman::initLibUtil();
        return HOFFMAN_OK;
    }
    HOFFMANC_CATCH_ERRS
}

const char * hoffman_err_msg(hoffman_c_context * context, const hoffman_c_context * read_context, unsigned int * n)
{
    if (context)
        context->last_err_code = HOFFMAN_OK;
    if (read_context->last_err && read_context->last_err_code != HOFFMAN_OK) {
        if (n)
            *n = read_context->last_err->size();
        return read_context->last_err->c_str();
    }
    hoffman_set_err_msg(context, HOFFMAN_ERR_UNKNOWN, "No error message");
    return nullptr;
}

hoffman_err hoffman_err_name(
    hoffman_c_context * context, const hoffman_c_context * read_context, hoffman_get_string_callback callback, void * user_data)
{
    if (context)
        context->last_err_code = HOFFMAN_OK;
    if (read_context->last_err_code != HOFFMAN_ERR_HOFFMAN_ERROR) {
        return hoffman_set_err_msg(context, HOFFMAN_ERR_UNKNOWN, "Last error was not a hoffman error");
    }
    return call_hoffman_get_string_callback(read_context->name, callback, user_data);
}

hoffman_err hoffman_err_info_msg(
    hoffman_c_context * context, const hoffman_c_context * read_context, hoffman_get_string_callback callback, void * user_data)
{
    if (context)
        context->last_err_code = HOFFMAN_OK;
    if (read_context->last_err_code != HOFFMAN_ERR_HOFFMAN_ERROR) {
        return hoffman_set_err_msg(context, HOFFMAN_ERR_UNKNOWN, "Last error was not a hoffman error");
    }
    return call_hoffman_get_string_callback(read_context->info->msg.str(), callback, user_data);
}

hoffman_err hoffman_err_code(const hoffman_c_context * read_context)
{
    return read_context->last_err_code;
}

// internal
hoffman_err call_hoffman_get_string_callback(const std::string_view str, hoffman_get_string_callback callback, void * user_data)
{
    callback(str.data(), str.size(), user_data);
    return HOFFMAN_OK;
}

hoffman_err hoffman_set_verbosity(hoffman_c_context * context, hoffman_verbosity level)
{
    if (context)
        context->last_err_code = HOFFMAN_OK;
    if (level > HOFFMAN_LVL_VOMIT || level < HOFFMAN_LVL_ERROR)
        return hoffman_set_err_msg(context, HOFFMAN_ERR_UNKNOWN, "Invalid verbosity level");
    try {
        hoffman::verbosity = static_cast<hoffman::Verbosity>(level);
    }
    HOFFMANC_CATCH_ERRS
}

} // extern "C"
