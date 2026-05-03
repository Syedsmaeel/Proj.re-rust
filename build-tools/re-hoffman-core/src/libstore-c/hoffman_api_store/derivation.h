#ifndef HOFFMAN_API_STORE_DERIVATION_H
#define HOFFMAN_API_STORE_DERIVATION_H
/**
 * @defgroup libstore_derivation Derivation
 * @ingroup libstore
 * @brief Derivation operations that don't require a Store
 * @{
 */
/** @file
 * @brief Derivation operations
 */

#include "hoffman_api_util.h"

#ifdef __cplusplus
extern "C" {
#endif
// cffi start

/** @brief Hoffman Derivation */
typedef struct hoffman_derivation hoffman_derivation;

/**
 * @brief Copy a `hoffman_derivation`
 *
 * @param[in] d the derivation to copy
 * @return a new `hoffman_derivation`
 */
hoffman_derivation * hoffman_derivation_clone(const hoffman_derivation * d);

/**
 * @brief Deallocate a `hoffman_derivation`
 *
 * Does not fail.
 * @param[in] drv the derivation to free
 */
void hoffman_derivation_free(hoffman_derivation * drv);

/**
 * @brief Gets the derivation as a JSON string
 *
 * @param[out] context Optional, stores error information
 * @param[in] drv The derivation
 * @param[in] callback Called with the JSON string
 * @param[in] userdata Arbitrary data passed to the callback
 */
hoffman_err hoffman_derivation_to_json(
    hoffman_c_context * context, const hoffman_derivation * drv, hoffman_get_string_callback callback, void * userdata);

// cffi end
#ifdef __cplusplus
}
#endif
/**
 * @}
 */
#endif // HOFFMAN_API_STORE_DERIVATION_H
