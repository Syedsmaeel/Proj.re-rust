#ifndef HOFFMAN_API_MAIN_H
#define HOFFMAN_API_MAIN_H
/**
 * @defgroup libmain libmain
 * @brief C bindings for hoffman libmain
 *
 * libmain has misc utilities for CLI commands
 * @{
 */
/** @file
 * @brief Main entry for the libmain C bindings
 */

#include "hoffman_api_util.h"
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif
// cffi start

/**
 * @brief Loads the plugins specified in Hoffman's plugin-files setting.
 *
 * Call this once, after calling your desired init functions and setting
 * relevant settings.
 *
 * @param[out] context Optional, stores error information
 * @return HOFFMAN_OK if the initialization was successful, an error code otherwise.
 */
hoffman_err hoffman_init_plugins(hoffman_c_context * context);

/**
 * @brief Sets the log format
 *
 * @param[out] context Optional, stores error information
 * @param[in] format The string name of the format.
 */
hoffman_err hoffman_set_log_format(hoffman_c_context * context, const char * format);

// cffi end
#ifdef __cplusplus
}
#endif
/**
 * @}
 */
#endif // HOFFMAN_API_MAIN_H
