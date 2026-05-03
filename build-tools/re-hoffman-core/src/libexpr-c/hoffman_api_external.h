#ifndef HOFFMAN_API_EXTERNAL_H
#define HOFFMAN_API_EXTERNAL_H
/** @ingroup libexpr
 * @addtogroup Externals
 * @brief Externals let Hoffman expressions work with foreign values that aren't part of the normal Hoffman value data model
 * @{
 */
/** @file
 * @brief libexpr C bindings dealing with external values
 * @see Externals
 */

#include "hoffman_api_expr.h"
#include "hoffman_api_util.h"
#include "hoffman_api_value.h"

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif
// cffi start

/**
 * @brief Represents a string owned by the Hoffman language evaluator.
 * @see hoffman_set_owned_string
 */
typedef struct hoffman_string_return hoffman_string_return;
/**
 * @brief Wraps a stream that can output multiple string pieces.
 */
typedef struct hoffman_printer hoffman_printer;
/**
 * @brief A list of string context items
 */
typedef struct hoffman_string_context hoffman_string_context;

/**
 * @brief Sets the contents of a hoffman_string_return
 *
 * Copies the passed string.
 * @param[out] str the hoffman_string_return to write to
 * @param[in]  c   The string to copy
 */
void hoffman_set_string_return(hoffman_string_return * str, const char * c);

/**
 * Print to the hoffman_printer
 *
 * @param[out] context Optional, stores error information
 * @param[out] printer The hoffman_printer to print to
 * @param[in] str The string to print
 * @returns HOFFMAN_OK if everything worked
 */
hoffman_err hoffman_external_print(hoffman_c_context * context, hoffman_printer * printer, const char * str);

/**
 * Add string context to the hoffman_string_context object
 * @param[out] context Optional, stores error information
 * @param[out] string_context The hoffman_string_context to add to
 * @param[in] c The context string to add
 * @returns HOFFMAN_OK if everything worked
 */
hoffman_err hoffman_external_add_string_context(hoffman_c_context * context, hoffman_string_context * string_context, const char * c);

/**
 * @brief Definition for a class of external values
 *
 * Create and implement one of these, then pass it to hoffman_create_external_value
 * Make sure to keep it alive while the external value lives.
 *
 * Optional functions can be set to NULL
 *
 * @see hoffman_create_external_value
 */
typedef struct HoffmanCExternalValueDesc
{
    /**
     * @brief Called when printing the external value
     *
     * @param[in] self the void* passed to hoffman_create_external_value
     * @param[out] printer The printer to print to, pass to hoffman_external_print
     */
    void (*print)(void * self, hoffman_printer * printer);
    /**
     * @brief Called on :t
     * @param[in] self the void* passed to hoffman_create_external_value
     * @param[out] res the return value
     */
    void (*showType)(void * self, hoffman_string_return * res);
    /**
     * @brief Called on `builtins.typeOf`
     * @param self the void* passed to hoffman_create_external_value
     * @param[out] res the return value
     */
    void (*typeOf)(void * self, hoffman_string_return * res);
    /**
     * @brief Called on "${str}" and builtins.toString.
     *
     * The latter with coerceMore=true
     * Optional, the default is to throw an error.
     * @param[in] self the void* passed to hoffman_create_external_value
     * @param[out] c writable string context for the resulting string
     * @param[in] coerceMore boolean, try to coerce to strings in more cases
     * instead of throwing an error
     * @param[in] copyToStore boolean, whether to copy referenced paths to store
     * or keep them as-is
     * @param[out] res the return value. Not touching this, or setting it to the
     * empty string, will make the conversion throw an error.
     */
    void (*coerceToString)(
        void * self, hoffman_string_context * c, int coerceMore, int copyToStore, hoffman_string_return * res);
    /**
     * @brief Try to compare two external values
     *
     * Optional, the default is always false.
     * If the other object was not a Hoffman C API external value, this comparison will
     * also return false
     * @param[in] self the void* passed to hoffman_create_external_value
     * @param[in] other the void* passed to the other object's
     * hoffman_create_external_value
     * @returns true if the objects are deemed to be equal
     */
    int (*equal)(void * self, void * other);
    /**
     * @brief Convert the external value to json
     *
     * Optional, the default is to throw an error
     * @param[in] self the void* passed to hoffman_create_external_value
     * @param[in] state The evaluator state
     * @param[in] strict boolean Whether to force the value before printing
     * @param[out] c writable string context for the resulting string
     * @param[in] copyToStore whether to copy referenced paths to store or keep
     * them as-is
     * @param[out] res the return value. Gets parsed as JSON. Not touching this,
     * or setting it to the empty string, will make the conversion throw an error.
     */
    void (*printValueAsJSON)(
        void * self, EvalState * state, bool strict, hoffman_string_context * c, bool copyToStore, hoffman_string_return * res);
    /**
     * @brief Convert the external value to XML
     *
     * Optional, the default is to throw an error
     * @todo The mechanisms for this call are incomplete. There are no C
     *       bindings to work with XML, pathsets and positions.
     *       This callback also has no test coverage.
     * @param[in] self the void* passed to hoffman_create_external_value
     * @param[in] state The evaluator state
     * @param[in] strict boolean Whether to force the value before printing
     * @param[in] location boolean Whether to include position information in the
     * xml
     * @param[out] doc XML document to output to
     * @param[out] c writable string context for the resulting string
     * @param[in,out] drvsSeen a path set to avoid duplicating derivations
     * @param[in] pos The position of the call.
     */
    void (*printValueAsXML)(
        void * self,
        EvalState * state,
        int strict,
        int location,
        void * doc,
        hoffman_string_context * c,
        void * drvsSeen,
        int pos);
} HoffmanCExternalValueDesc;

/**
 * @brief Create an external value, that can be given to hoffman_init_external
 *
 * Call hoffman_gc_decref() when you're done with the pointer.
 *
 * @param[out] context Optional, stores error information
 * @param[in] desc a HoffmanCExternalValueDesc, you should keep this alive as long
 * as the ExternalValue lives
 * @param[in] v the value to store
 * @returns external value, owned by the garbage collector
 * @see hoffman_init_external
 */
ExternalValue * hoffman_create_external_value(hoffman_c_context * context, HoffmanCExternalValueDesc * desc, void * v);

/**
 * @brief Extract the pointer from a Hoffman C API external value.
 * @param[out] context Optional, stores error information
 * @param[in] b The external value
 * @returns The pointer, valid while the external value is valid, or null if the external value was not from the Hoffman C
 * API.
 * @see hoffman_get_external
 */
void * hoffman_get_external_value_content(hoffman_c_context * context, ExternalValue * b);

// cffi end
#ifdef __cplusplus
}
#endif
/** @} */

#endif // HOFFMAN_API_EXTERNAL_H
