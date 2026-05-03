#include "hoffman_api_util.h"
#include "hoffman_api_util_internal.h"

#include "hoffman/main/plugin.hh"
#include "hoffman/main/loggers.hh"

extern "C" {

hoffman_err hoffman_init_plugins(hoffman_c_context * context)
{
    if (context)
        context->last_err_code = HOFFMAN_OK;
    try {
        hoffman::initPlugins();
    }
    HOFFMANC_CATCH_ERRS
}

hoffman_err hoffman_set_log_format(hoffman_c_context * context, const char * format)
{
    if (context)
        context->last_err_code = HOFFMAN_OK;
    if (format == nullptr)
        return hoffman_set_err_msg(context, HOFFMAN_ERR_UNKNOWN, "Log format is null");
    try {
        hoffman::setLogFormat(format);
    }
    HOFFMANC_CATCH_ERRS
}

} // extern "C"
