#ifndef HOFFMAN_API_VALUE_H
#define HOFFMAN_API_VALUE_H

/** @file
 * @brief libexpr C bindings dealing with values
 */

#include "hoffman_api_util.h"
#include "hoffman_api_store.h"

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif
// cffi start

/** @defgroup value Value
 * @ingroup libexpr
 * @brief hoffman_value type and core operations for working with Hoffman values
 * @see value_create
 * @see value_extract
 */

/** @defgroup value_create Value Creation
 * @ingroup libexpr
 * @brief Functions for allocating and initializing Hoffman values
 *
 * Values are usually created with `hoffman_alloc_value` followed by `hoffman_init_*` functions.
 * In primop callbacks, allocation is already done and only initialization is needed.
 */

/** @defgroup value_extract Value Extraction
 * @ingroup libexpr
 * @brief Functions for extracting data from Hoffman values
 */

/** @defgroup primops PrimOps and Builtins
 * @ingroup libexpr
 */

// Type definitions
/** @brief Represents the state of a Hoffman value
 *
 * Thunk values (HOFFMAN_TYPE_THUNK) change to their final, unchanging type when forced.
 *
 * @see https://hoffman.dev/manual/hoffman/latest/language/evaluation.html
 * @enum ValueType
 * @ingroup value
 */
typedef enum {
    /** Unevaluated expression
     *
     * Thunks often contain an expression and closure, but may contain other
     * representations too.
     *
     * Their state is mutable, unlike that of the other types.
     */
    HOFFMAN_TYPE_THUNK,
    /**
     * A 64 bit signed integer.
     */
    HOFFMAN_TYPE_INT,
    /** @brief IEEE 754 double precision floating point number
     * @see https://hoffman.dev/manual/hoffman/latest/language/types.html#type-float
     */
    HOFFMAN_TYPE_FLOAT,
    /** @brief Boolean true or false value
     * @see https://hoffman.dev/manual/hoffman/latest/language/types.html#type-bool
     */
    HOFFMAN_TYPE_BOOL,
    /** @brief String value with context
     *
     * String content may contain arbitrary bytes, not necessarily UTF-8.
     * @see https://hoffman.dev/manual/hoffman/latest/language/types.html#type-string
     */
    HOFFMAN_TYPE_STRING,
    /** @brief Filesystem path
     * @see https://hoffman.dev/manual/hoffman/latest/language/types.html#type-path
     */
    HOFFMAN_TYPE_PATH,
    /** @brief Null value
     * @see https://hoffman.dev/manual/hoffman/latest/language/types.html#type-null
     */
    HOFFMAN_TYPE_NULL,
    /** @brief Attribute set (key-value mapping)
     * @see https://hoffman.dev/manual/hoffman/latest/language/types.html#type-attrs
     */
    HOFFMAN_TYPE_ATTRS,
    /** @brief Ordered list of values
     * @see https://hoffman.dev/manual/hoffman/latest/language/types.html#type-list
     */
    HOFFMAN_TYPE_LIST,
    /** @brief Function (lambda or builtin)
     * @see https://hoffman.dev/manual/hoffman/latest/language/types.html#type-function
     */
    HOFFMAN_TYPE_FUNCTION,
    /** @brief External value from C++ plugins or C API
     * @see Externals
     */
    HOFFMAN_TYPE_EXTERNAL,
    HOFFMAN_TYPE_FAILED,
} ValueType;

// forward declarations
typedef struct hoffman_value hoffman_value;
typedef struct EvalState EvalState;

/** @deprecated Use hoffman_value instead */
[[deprecated("use hoffman_value instead")]] typedef hoffman_value Value;

// type defs
/** @brief Stores an under-construction set of bindings
 * @ingroup value_create
 *
 * Each builder can only be used once. After calling hoffman_make_attrs(), the builder
 * becomes invalid and must not be used again. Call hoffman_bindings_builder_free() to release it.
 *
 * Typical usage pattern:
 * 1. Create with hoffman_make_bindings_builder()
 * 2. Insert attributes with hoffman_bindings_builder_insert()
 * 3. Create final attribute set with hoffman_make_attrs()
 * 4. Free builder with hoffman_bindings_builder_free()
 *
 * @struct BindingsBuilder
 * @see hoffman_make_bindings_builder, hoffman_bindings_builder_free, hoffman_make_attrs
 * @see hoffman_bindings_builder_insert
 */
typedef struct BindingsBuilder BindingsBuilder;

/** @brief Stores an under-construction list
 * @ingroup value_create
 *
 * Each builder can only be used once. After calling hoffman_make_list(), the builder
 * becomes invalid and must not be used again. Call hoffman_list_builder_free() to release it.
 *
 * Typical usage pattern:
 * 1. Create with hoffman_make_list_builder()
 * 2. Insert elements with hoffman_list_builder_insert()
 * 3. Create final list with hoffman_make_list()
 * 4. Free builder with hoffman_list_builder_free()
 *
 * @struct ListBuilder
 * @see hoffman_make_list_builder, hoffman_list_builder_free, hoffman_make_list
 * @see hoffman_list_builder_insert
 */
typedef struct ListBuilder ListBuilder;

/** @brief PrimOp function
 * @ingroup primops
 *
 * Can be released with hoffman_gc_decref() when necessary.
 * @struct PrimOp
 * @see hoffman_alloc_primop, hoffman_init_primop, hoffman_register_primop
 */
typedef struct PrimOp PrimOp;
/** @brief External Value
 * @ingroup Externals
 *
 * Can be released with hoffman_gc_decref() when necessary.
 * @struct ExternalValue
 * @see hoffman_create_external_value, hoffman_init_external, hoffman_get_external
 */
typedef struct ExternalValue ExternalValue;

/** @brief String without placeholders, and realised store paths
 * @struct hoffman_realised_string
 * @see hoffman_string_realise, hoffman_realised_string_free
 */
typedef struct hoffman_realised_string hoffman_realised_string;

/** @brief Function pointer for primops
 * @ingroup primops
 *
 * When you want to return an error, call hoffman_set_err_msg(context, HOFFMAN_ERR_UNKNOWN, "your error message here").
 *
 * @param[in] user_data Arbitrary data that was initially supplied to hoffman_alloc_primop
 * @param[out] context Stores error information.
 * @param[in] state Evaluator state
 * @param[in] args list of arguments. Note that these can be thunks and should be forced using hoffman_value_force before
 * use.
 * @param[out] ret return value
 * @see hoffman_alloc_primop, hoffman_init_primop
 */
typedef void (*PrimOpFun)(
    void * user_data, hoffman_c_context * context, EvalState * state, hoffman_value ** args, hoffman_value * ret);

/** @brief Allocate a PrimOp
 * @ingroup primops
 *
 * Call hoffman_gc_decref() when you're done with the returned PrimOp.
 *
 * @param[out] context Optional, stores error information
 * @param[in] fun callback
 * @param[in] arity expected number of function arguments
 * @param[in] name function name
 * @param[in] args array of argument names, NULL-terminated
 * @param[in] doc optional, documentation for this primop
 * @param[in] user_data optional, arbitrary data, passed to the callback when it's called
 * @return primop, or null in case of errors
 * @see hoffman_init_primop
 */
PrimOp * hoffman_alloc_primop(
    hoffman_c_context * context,
    PrimOpFun fun,
    int arity,
    const char * name,
    const char ** args,
    const char * doc,
    void * user_data);

/** @brief add a primop to the `builtins` attribute set
 * @ingroup primops
 *
 * Only applies to States created after this call.
 *
 * Moves your PrimOp content into the global evaluator registry, meaning
 * your input PrimOp pointer becomes invalid. The PrimOp must not be used
 * with hoffman_init_primop() before or after this call, as this would cause
 * undefined behavior.
 * You must call hoffman_gc_decref() on the original PrimOp pointer
 * after this call to release your reference.
 *
 * @param[out] context Optional, stores error information
 * @param[in] primOp PrimOp to register
 * @return error code, HOFFMAN_OK on success
 */
hoffman_err hoffman_register_primop(hoffman_c_context * context, PrimOp * primOp);

// Function prototypes

/** @brief Allocate a Hoffman value
 * @ingroup value_create
 *
 * Call hoffman_value_decref() when you're done with the pointer
 * @param[out] context Optional, stores error information
 * @param[in] state hoffman evaluator state
 * @return value, or null in case of errors
 */
hoffman_value * hoffman_alloc_value(hoffman_c_context * context, EvalState * state);

/**
 * @brief Increment the garbage collector reference counter for the given `hoffman_value`.
 * @ingroup value
 *
 * The Hoffman language evaluator C API keeps track of alive objects by reference counting.
 * When you're done with a refcounted pointer, call hoffman_value_decref().
 *
 * @param[out] context Optional, stores error information
 * @param[in] value The object to keep alive
 */
hoffman_err hoffman_value_incref(hoffman_c_context * context, hoffman_value * value);

/**
 * @brief Decrement the garbage collector reference counter for the given object
 * @ingroup value
 *
 * When the counter reaches zero, the `hoffman_value` object becomes invalid.
 * The data referenced by `hoffman_value` may not be deallocated until the memory
 * garbage collector has run, but deallocation is not guaranteed.
 *
 * @param[out] context Optional, stores error information
 * @param[in] value The object to stop referencing
 */
hoffman_err hoffman_value_decref(hoffman_c_context * context, hoffman_value * value);

/** @brief Get value type
 * @ingroup value_extract
 * @param[out] context Optional, stores error information
 * @param[in] value Hoffman value to inspect
 * @return type of hoffman value
 */
ValueType hoffman_get_type(hoffman_c_context * context, const hoffman_value * value);

/** @brief Get type name of value as defined in the evaluator
 * @ingroup value_extract
 * @param[out] context Optional, stores error information
 * @param[in] value Hoffman value to inspect
 * @return type name string, free with free()
 */
const char * hoffman_get_typename(hoffman_c_context * context, const hoffman_value * value);

/** @brief Get boolean value
 * @ingroup value_extract
 * @param[out] context Optional, stores error information
 * @param[in] value Hoffman value to inspect
 * @return true or false, error info via context
 */
bool hoffman_get_bool(hoffman_c_context * context, const hoffman_value * value);

/** @brief Get the raw string
 * @ingroup value_extract
 *
 * This may contain placeholders.
 *
 * @param[out] context Optional, stores error information
 * @param[in] value Hoffman value to inspect
 * @param[in] callback Called with the string value.
 * @param[in] user_data optional, arbitrary data, passed to the callback when it's called.
 * @return error code, HOFFMAN_OK on success.
 */
hoffman_err
hoffman_get_string(hoffman_c_context * context, const hoffman_value * value, hoffman_get_string_callback callback, void * user_data);

/** @brief Get path as string
 * @ingroup value_extract
 * @param[out] context Optional, stores error information
 * @param[in] value Hoffman value to inspect
 * @return string valid while value is valid, NULL in case of error
 */
const char * hoffman_get_path_string(hoffman_c_context * context, const hoffman_value * value);

/** @brief Get the length of a list
 * @ingroup value_extract
 * @param[out] context Optional, stores error information
 * @param[in] value Hoffman value to inspect
 * @return length of list, error info via context
 */
unsigned int hoffman_get_list_size(hoffman_c_context * context, const hoffman_value * value);

/** @brief Get the element count of an attrset
 * @ingroup value_extract
 * @param[out] context Optional, stores error information
 * @param[in] value Hoffman value to inspect
 * @return attrset element count, error info via context
 */
unsigned int hoffman_get_attrs_size(hoffman_c_context * context, const hoffman_value * value);

/** @brief Get float value in 64 bits
 * @ingroup value_extract
 * @param[out] context Optional, stores error information
 * @param[in] value Hoffman value to inspect
 * @return float contents, error info via context
 */
double hoffman_get_float(hoffman_c_context * context, const hoffman_value * value);

/** @brief Get int value
 * @ingroup value_extract
 * @param[out] context Optional, stores error information
 * @param[in] value Hoffman value to inspect
 * @return int contents, error info via context
 */
int64_t hoffman_get_int(hoffman_c_context * context, const hoffman_value * value);

/** @brief Get external reference
 * @ingroup value_extract
 * @param[out] context Optional, stores error information
 * @param[in] value Hoffman value to inspect
 * @return reference valid while value is valid. Call hoffman_gc_incref() if you need it to live longer, then only in that
 * case call hoffman_gc_decref() when done. NULL in case of error
 */
ExternalValue * hoffman_get_external(hoffman_c_context * context, hoffman_value * value);

/** @brief Get the ix'th element of a list
 * @ingroup value_extract
 *
 * Call hoffman_value_decref() when you're done with the pointer
 * @param[out] context Optional, stores error information
 * @param[in] value Hoffman value to inspect
 * @param[in] state hoffman evaluator state
 * @param[in] ix list element to get
 * @return value, NULL in case of errors
 */
hoffman_value * hoffman_get_list_byidx(hoffman_c_context * context, const hoffman_value * value, EvalState * state, unsigned int ix);

/** @brief Get the ix'th element of a list without forcing evaluation of the element
 * @ingroup value_extract
 *
 * Returns the list element without forcing its evaluation, allowing access to lazy values.
 * The list value itself must already be evaluated.
 *
 * Call hoffman_value_decref() when you're done with the pointer
 * @param[out] context Optional, stores error information
 * @param[in] value Hoffman value to inspect (must be an evaluated list)
 * @param[in] state hoffman evaluator state
 * @param[in] ix list element to get
 * @return value, NULL in case of errors
 */
hoffman_value *
hoffman_get_list_byidx_lazy(hoffman_c_context * context, const hoffman_value * value, EvalState * state, unsigned int ix);

/** @brief Get an attr by name
 * @ingroup value_extract
 *
 * Call hoffman_value_decref() when you're done with the pointer
 * @param[out] context Optional, stores error information
 * @param[in] value Hoffman value to inspect
 * @param[in] state hoffman evaluator state
 * @param[in] name attribute name
 * @return value, NULL in case of errors
 */
hoffman_value * hoffman_get_attr_byname(hoffman_c_context * context, const hoffman_value * value, EvalState * state, const char * name);

/** @brief Get an attribute value by attribute name, without forcing evaluation of the attribute's value
 * @ingroup value_extract
 *
 * Returns the attribute value without forcing its evaluation, allowing access to lazy values.
 * The attribute set value itself must already be evaluated.
 *
 * Call hoffman_value_decref() when you're done with the pointer
 * @param[out] context Optional, stores error information
 * @param[in] value Hoffman value to inspect (must be an evaluated attribute set)
 * @param[in] state hoffman evaluator state
 * @param[in] name attribute name
 * @return value, NULL in case of errors
 */
hoffman_value *
hoffman_get_attr_byname_lazy(hoffman_c_context * context, const hoffman_value * value, EvalState * state, const char * name);

/** @brief Check if an attribute name exists on a value
 * @ingroup value_extract
 * @param[out] context Optional, stores error information
 * @param[in] value Hoffman value to inspect
 * @param[in] state hoffman evaluator state
 * @param[in] name attribute name
 * @return value, error info via context
 */
bool hoffman_has_attr_byname(hoffman_c_context * context, const hoffman_value * value, EvalState * state, const char * name);

/** @brief Get an attribute by index
 * @ingroup value_extract
 *
 * Also gives you the name.
 *
 * Attributes are returned in an unspecified order which is NOT suitable for
 * reproducible operations. In Hoffman's domain, reproducibility is paramount. The caller
 * is responsible for sorting the attributes or storing them in an ordered map to
 * ensure deterministic behavior in your application.
 *
 * @note When Hoffman does sort attributes, which it does for virtually all intermediate
 * operations and outputs, it uses byte-wise lexicographic order (equivalent to
 * lexicographic order by Unicode scalar value for valid UTF-8). We recommend
 * applying this same ordering for consistency.
 *
 * Call hoffman_value_decref() when you're done with the pointer
 * @param[out] context Optional, stores error information
 * @param[in] value Hoffman value to inspect
 * @param[in] state hoffman evaluator state
 * @param[in] i attribute index
 * @param[out] name will store a pointer to the attribute name, valid until state is freed
 * @return value, NULL in case of errors
 */
hoffman_value *
hoffman_get_attr_byidx(hoffman_c_context * context, hoffman_value * value, EvalState * state, unsigned int i, const char ** name);

/** @brief Get an attribute by index, without forcing evaluation of the attribute's value
 * @ingroup value_extract
 *
 * Also gives you the name.
 *
 * Returns the attribute value without forcing its evaluation, allowing access to lazy values.
 * The attribute set value itself must already have been evaluated.
 *
 * Attributes are returned in an unspecified order which is NOT suitable for
 * reproducible operations. In Hoffman's domain, reproducibility is paramount. The caller
 * is responsible for sorting the attributes or storing them in an ordered map to
 * ensure deterministic behavior in your application.
 *
 * @note When Hoffman does sort attributes, which it does for virtually all intermediate
 * operations and outputs, it uses byte-wise lexicographic order (equivalent to
 * lexicographic order by Unicode scalar value for valid UTF-8). We recommend
 * applying this same ordering for consistency.
 *
 * Call hoffman_value_decref() when you're done with the pointer
 * @param[out] context Optional, stores error information
 * @param[in] value Hoffman value to inspect (must be an evaluated attribute set)
 * @param[in] state hoffman evaluator state
 * @param[in] i attribute index
 * @param[out] name will store a pointer to the attribute name, valid until state is freed
 * @return value, NULL in case of errors
 */
hoffman_value * hoffman_get_attr_byidx_lazy(
    hoffman_c_context * context, hoffman_value * value, EvalState * state, unsigned int i, const char ** name);

/** @brief Get an attribute name by index
 * @ingroup value_extract
 *
 * Returns the attribute name without forcing evaluation of the attribute's value.
 *
 * Attributes are returned in an unspecified order which is NOT suitable for
 * reproducible operations. In Hoffman's domain, reproducibility is paramount. The caller
 * is responsible for sorting the attributes or storing them in an ordered map to
 * ensure deterministic behavior in your application.
 *
 * @note When Hoffman does sort attributes, which it does for virtually all intermediate
 * operations and outputs, it uses byte-wise lexicographic order (equivalent to
 * lexicographic order by Unicode scalar value for valid UTF-8). We recommend
 * applying this same ordering for consistency.
 *
 * @param[out] context Optional, stores error information
 * @param[in] value Hoffman value to inspect
 * @param[in] state hoffman evaluator state
 * @param[in] i attribute index
 * @return name string valid until state is freed, NULL in case of errors
 */
const char * hoffman_get_attr_name_byidx(hoffman_c_context * context, hoffman_value * value, EvalState * state, unsigned int i);

/** @name Initializers
 *
 * Values are typically "returned" by initializing already allocated memory that serves as the return value.
 * For this reason, the construction of values is not tied their allocation.
 * Hoffman is a language with immutable values. Respect this property by only initializing Values once; and only initialize
 * Values that are meant to be initialized by you. Failing to adhere to these rules may lead to undefined behavior.
 */
/**@{*/
/** @brief Set boolean value
 * @ingroup value_create
 * @param[out] context Optional, stores error information
 * @param[out] value Hoffman value to modify
 * @param[in] b the boolean value
 * @return error code, HOFFMAN_OK on success.
 */
hoffman_err hoffman_init_bool(hoffman_c_context * context, hoffman_value * value, bool b);

/** @brief Set a string
 * @ingroup value_create
 * @param[out] context Optional, stores error information
 * @param[out] value Hoffman value to modify
 * @param[in] str the string, copied
 * @return error code, HOFFMAN_OK on success.
 */
hoffman_err hoffman_init_string(hoffman_c_context * context, hoffman_value * value, const char * str);

/** @brief Set a path
 * @ingroup value_create
 * @param[out] context Optional, stores error information
 * @param[out] value Hoffman value to modify
 * @param[in] str the path string, copied
 * @return error code, HOFFMAN_OK on success.
 */
hoffman_err hoffman_init_path_string(hoffman_c_context * context, EvalState * s, hoffman_value * value, const char * str);

/** @brief Set a float
 * @ingroup value_create
 * @param[out] context Optional, stores error information
 * @param[out] value Hoffman value to modify
 * @param[in] d the float, 64-bits
 * @return error code, HOFFMAN_OK on success.
 */
hoffman_err hoffman_init_float(hoffman_c_context * context, hoffman_value * value, double d);

/** @brief Set an int
 * @ingroup value_create
 * @param[out] context Optional, stores error information
 * @param[out] value Hoffman value to modify
 * @param[in] i the int
 * @return error code, HOFFMAN_OK on success.
 */

hoffman_err hoffman_init_int(hoffman_c_context * context, hoffman_value * value, int64_t i);
/** @brief Set null
 * @ingroup value_create
 * @param[out] context Optional, stores error information
 * @param[out] value Hoffman value to modify
 * @return error code, HOFFMAN_OK on success.
 */
hoffman_err hoffman_init_null(hoffman_c_context * context, hoffman_value * value);

/** @brief Set the value to a thunk that will perform a function application when needed.
 * @ingroup value_create
 *
 * Thunks may be put into attribute sets and lists to perform some computation lazily; on demand.
 * However, note that in some places, a thunk must not be returned, such as in the return value of a PrimOp.
 * In such cases, you may use hoffman_value_call() instead (but note the different argument order).
 *
 * @param[out] context Optional, stores error information
 * @param[out] value Hoffman value to modify
 * @param[in] fn function to call
 * @param[in] arg argument to pass
 * @return error code, HOFFMAN_OK on successful initialization.
 * @see hoffman_value_call() for a similar function that performs the call immediately and only stores the return value.
 *      Note the different argument order.
 */
hoffman_err hoffman_init_apply(hoffman_c_context * context, hoffman_value * value, hoffman_value * fn, hoffman_value * arg);

/** @brief Set an external value
 * @ingroup value_create
 * @param[out] context Optional, stores error information
 * @param[out] value Hoffman value to modify
 * @param[in] val the external value to set. Will be GC-referenced by the value.
 * @return error code, HOFFMAN_OK on success.
 */
hoffman_err hoffman_init_external(hoffman_c_context * context, hoffman_value * value, ExternalValue * val);

/** @brief Create a list from a list builder
 * @ingroup value_create
 *
 * After this call, the list builder becomes invalid and cannot be used again.
 * The only necessary next step is to free it with hoffman_list_builder_free().
 *
 * @param[out] context Optional, stores error information
 * @param[in] list_builder list builder to use
 * @param[out] value Hoffman value to modify
 * @return error code, HOFFMAN_OK on success.
 * @see hoffman_list_builder_free
 */
hoffman_err hoffman_make_list(hoffman_c_context * context, ListBuilder * list_builder, hoffman_value * value);

/** @brief Create a list builder
 * @ingroup value_create
 * @param[out] context Optional, stores error information
 * @param[in] state hoffman evaluator state
 * @param[in] capacity how many bindings you'll add. Don't exceed.
 * @return list builder. Call hoffman_list_builder_free() when you're done.
 */
ListBuilder * hoffman_make_list_builder(hoffman_c_context * context, EvalState * state, size_t capacity);

/** @brief Insert bindings into a builder
 * @param[out] context Optional, stores error information
 * @param[in] list_builder ListBuilder to insert into
 * @param[in] index index to manipulate
 * @param[in] value value to insert
 * @return error code, HOFFMAN_OK on success.
 */
hoffman_err
hoffman_list_builder_insert(hoffman_c_context * context, ListBuilder * list_builder, unsigned int index, hoffman_value * value);

/** @brief Free a list builder
 *
 * Does not fail.
 * @param[in] list_builder The builder to free.
 */
void hoffman_list_builder_free(ListBuilder * list_builder);

/** @brief Create an attribute set from a bindings builder
 * @ingroup value_create
 *
 * After this call, the bindings builder becomes invalid and cannot be used again.
 * The only necessary next step is to free it with hoffman_bindings_builder_free().
 *
 * @param[out] context Optional, stores error information
 * @param[out] value Hoffman value to modify
 * @param[in] b bindings builder to use
 * @return error code, HOFFMAN_OK on success.
 * @see hoffman_bindings_builder_free
 */
hoffman_err hoffman_make_attrs(hoffman_c_context * context, hoffman_value * value, BindingsBuilder * b);

/** @brief Set primop
 * @ingroup value_create
 * @param[out] context Optional, stores error information
 * @param[out] value Hoffman value to modify
 * @param[in] op primop, will be gc-referenced by the value
 * @see hoffman_alloc_primop
 * @return error code, HOFFMAN_OK on success.
 */
hoffman_err hoffman_init_primop(hoffman_c_context * context, hoffman_value * value, PrimOp * op);
/** @brief Copy from another value
 * @ingroup value_create
 * @param[out] context Optional, stores error information
 * @param[out] value Hoffman value to modify
 * @param[in] source value to copy from
 * @return error code, HOFFMAN_OK on success.
 */
hoffman_err hoffman_copy_value(hoffman_c_context * context, hoffman_value * value, const hoffman_value * source);
/**@}*/

/** @brief Create a bindings builder
 * @param[out] context Optional, stores error information
 * @param[in] state hoffman evaluator state
 * @param[in] capacity how many bindings you'll add. Don't exceed.
 * @return bindings builder. Call hoffman_bindings_builder_free() when you're done.
 */
BindingsBuilder * hoffman_make_bindings_builder(hoffman_c_context * context, EvalState * state, size_t capacity);

/** @brief Insert bindings into a builder
 * @param[out] context Optional, stores error information
 * @param[in] builder BindingsBuilder to insert into
 * @param[in] name attribute name, only used for the duration of the call.
 * @param[in] value value to give the binding
 * @return error code, HOFFMAN_OK on success.
 */
hoffman_err
hoffman_bindings_builder_insert(hoffman_c_context * context, BindingsBuilder * builder, const char * name, hoffman_value * value);

/** @brief Free a bindings builder
 *
 * Does not fail.
 * @param[in] builder the builder to free
 */
void hoffman_bindings_builder_free(BindingsBuilder * builder);

/** @brief Realise a string context.
 *
 * This will
 *  - realise the store paths referenced by the string's context, and
 *  - perform the replacement of placeholders.
 *  - create temporary garbage collection roots for the store paths, for
 *    the lifetime of the current process.
 *  - log to stderr
 *
 * @param[out] context Optional, stores error information
 * @param[in] value Hoffman value, which must be a string
 * @param[in] state Hoffman evaluator state
 * @param[in] isIFD If true, disallow derivation outputs if setting `allow-import-from-derivation` is false.
                    You should set this to true when this call is part of a primop.
                    You should set this to false when building for your application's purpose.
 * @return NULL if failed, or a new hoffman_realised_string, which must be freed with hoffman_realised_string_free
 */
hoffman_realised_string * hoffman_string_realise(hoffman_c_context * context, EvalState * state, hoffman_value * value, bool isIFD);

/** @brief Start of the string
 * @param[in] realised_string
 * @return pointer to the start of the string, valid until realised_string is freed. It may not be null-terminated.
 */
const char * hoffman_realised_string_get_buffer_start(hoffman_realised_string * realised_string);

/** @brief Length of the string
 * @param[in] realised_string
 * @return length of the string in bytes
 */
size_t hoffman_realised_string_get_buffer_size(hoffman_realised_string * realised_string);

/** @brief Number of realised store paths
 * @param[in] realised_string
 * @return number of realised store paths that were referenced by the string via its context
 */
size_t hoffman_realised_string_get_store_path_count(hoffman_realised_string * realised_string);

/** @brief Get a store path. The store paths are stored in an arbitrary order.
 * @param[in] realised_string
 * @param[in] index index of the store path, must be less than the count
 * @return store path valid until realised_string is freed
 */
const StorePath * hoffman_realised_string_get_store_path(hoffman_realised_string * realised_string, size_t index);

/** @brief Free a realised string
 * @param[in] realised_string
 */
void hoffman_realised_string_free(hoffman_realised_string * realised_string);

// cffi end
#ifdef __cplusplus
}
#endif

#endif // HOFFMAN_API_VALUE_H
