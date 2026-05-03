#ifndef HOFFMAN_API_FETCHERS_H
#define HOFFMAN_API_FETCHERS_H
/** @defgroup libfetchers libfetchers
 * @brief Bindings to the Hoffman fetchers library
 * @{
 */
/** @file
 * @brief Main entry for the libfetchers C bindings
 */

#include "hoffman_api_util.h"

#ifdef __cplusplus
extern "C" {
#endif
// cffi start

// Type definitions
/**
 * @brief Shared settings object
 */
typedef struct hoffman_fetchers_settings hoffman_fetchers_settings;

hoffman_fetchers_settings * hoffman_fetchers_settings_new(hoffman_c_context * context);

void hoffman_fetchers_settings_free(hoffman_fetchers_settings * settings);

#ifdef __cplusplus
} // extern "C"
#endif

#endif // HOFFMAN_API_FETCHERS_H