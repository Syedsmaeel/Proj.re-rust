#ifndef HOFFMAN_API_EXPR_H
#define HOFFMAN_API_EXPR_H
/** @defgroup libexpr libexpr
 * @brief Bindings to the Hoffman language evaluator
 *
 * See *[Embedding the Hoffman Evaluator](@ref hoffman_evaluator_example)* for an example.
 */
/** @file
 * @brief Main entry for the libexpr C bindings
 */
/** @defgroup libexpr_init Initialization
 * @ingroup libexpr
 * @{
 */

#include "hoffman_api_store.h"
#include "hoffman_api_util.h"
#include <stddef.h>

#ifndef __has_c_attribute
#  define __has_c_attribute(x) 0
#endif

#if __has_c_attribute(deprecated)
#  define HOFFMAN_DEPRECATED(msg) [[deprecated(msg)]]
#else
#  define HOFFMAN_DEPRECATED(msg)
#endif

#ifdef __cplusplus
extern "C" {
#endif
// cffi start

// Type definitions
/**
 * @brief Builder for EvalState
 */
typedef struct hoffman_eval_state_builder hoffman_eval_state_builder;

/**
 * @brief Represents a state of the Hoffman language evaluator.
 *
 * Multiple states can be created for multi-threaded
 * operation.
 * @struct EvalState
 * @see hoffman_state_create
 */
typedef struct EvalState EvalState; // hoffman::EvalState

/** @} */

/** @brief A Hoffman language value, or thunk that may evaluate to a value.
 * @ingroup value
 *
 * Values are the primary objects manipulated in the Hoffman language.
 * They are considered to be immutable from a user's perspective, but the process of evaluating a value changes its
 * ValueType if it was a thunk. After a value has been evaluated, its ValueType does not change.
 *
 * Evaluation in this context refers to the process of evaluating a single value object, also called "forcing" the
 * value; see `hoffman_value_force`.
 *
 * The evaluator manages its own memory, but your use of the C API must follow the reference counting rules.
 *
 * @struct hoffman_value
 * @see value_create, value_extract
 * @see hoffman_value_incref, hoffman_value_decref
 */
typedef struct hoffman_value hoffman_value;
HOFFMAN_DEPRECATED("use hoffman_value instead") typedef hoffman_value Value;

// Function prototypes
/**
 * @brief Initialize the Hoffman language evaluator.
 * @ingroup libexpr_init
 *
 * This function must be called at least once,
 * at some point before constructing a EvalState for the first time.
 * This function can be called multiple times, and is idempotent.
 *
 * @param[out] context Optional, stores error information
 * @return HOFFMAN_OK if the initialization was successful, an error code otherwise.
 */
hoffman_err hoffman_libexpr_init(hoffman_c_context * context);

/**
 * @brief Parses and evaluates a Hoffman expression from a string.
 * @ingroup value_create
 *
 * @param[out] context Optional, stores error information
 * @param[in] state The state of the evaluation.
 * @param[in] expr The Hoffman expression to parse.
 * @param[in] path The file path to associate with the expression.
 * This is required for expressions that contain relative paths (such as `./.`) that are resolved relative to the given
 * directory.
 * @param[out] value The result of the evaluation. You must allocate this
 * yourself.
 * @return HOFFMAN_OK if the evaluation was successful, an error code otherwise.
 */
hoffman_err hoffman_expr_eval_from_string(
    hoffman_c_context * context, EvalState * state, const char * expr, const char * path, hoffman_value * value);

/**
 * @brief Calls a Hoffman function with an argument.
 * @ingroup value_create
 *
 * @param[out] context Optional, stores error information
 * @param[in] state The state of the evaluation.
 * @param[in] fn The Hoffman function to call.
 * @param[in] arg The argument to pass to the function.
 * @param[out] value The result of the function call.
 * @return HOFFMAN_OK if the function call was successful, an error code otherwise.
 * @see hoffman_init_apply() for a similar function that does not performs the call immediately, but stores it as a thunk.
 *      Note the different argument order.
 */
hoffman_err hoffman_value_call(hoffman_c_context * context, EvalState * state, hoffman_value * fn, hoffman_value * arg, hoffman_value * value);

/**
 * @brief Calls a Hoffman function with multiple arguments.
 * @ingroup value_create
 *
 * Technically these are functions that return functions. It is common for Hoffman
 * functions to be curried, so this function is useful for calling them.
 *
 * @param[out] context Optional, stores error information
 * @param[in] state The state of the evaluation.
 * @param[in] fn The Hoffman function to call.
 * @param[in] nargs The number of arguments.
 * @param[in] args The arguments to pass to the function.
 * @param[out] value The result of the function call.
 *
 * @see hoffman_value_call     For the single argument primitive.
 * @see HOFFMAN_VALUE_CALL           For a macro that wraps this function for convenience.
 */
hoffman_err hoffman_value_call_multi(
    hoffman_c_context * context, EvalState * state, hoffman_value * fn, size_t nargs, hoffman_value ** args, hoffman_value * value);

/**
 * @brief Calls a Hoffman function with multiple arguments.
 * @ingroup value_create
 *
 * Technically these are functions that return functions. It is common for Hoffman
 * functions to be curried, so this function is useful for calling them.
 *
 * @def HOFFMAN_VALUE_CALL
 * @param[out] context Optional, stores error information
 * @param[in] state The state of the evaluation.
 * @param[out] value The result of the function call.
 * @param[in] fn The Hoffman function to call.
 * @param[in] ... The arguments to pass to the function.
 *
 * @see hoffman_value_call_multi
 */
#define HOFFMAN_VALUE_CALL(context, state, value, fn, ...)                      \
    do {                                                                    \
        hoffman_value * args_array[] = {__VA_ARGS__};                           \
        size_t nargs = sizeof(args_array) / sizeof(args_array[0]);          \
        hoffman_value_call_multi(context, state, fn, nargs, args_array, value); \
    } while (0)

/**
 * @brief Forces the evaluation of a Hoffman value.
 * @ingroup value_create
 *
 * The Hoffman interpreter is lazy, and not-yet-evaluated values can be
 * of type HOFFMAN_TYPE_THUNK instead of their actual value.
 *
 * This function mutates such a `hoffman_value`, so that, if successful, it has its final type.
 *
 * @param[out] context Optional, stores error information
 * @param[in] state The state of the evaluation.
 * @param[in,out] value The Hoffman value to force.
 * @post value is not of type HOFFMAN_TYPE_THUNK
 * @return HOFFMAN_OK if the force operation was successful, an error code
 * otherwise.
 */
hoffman_err hoffman_value_force(hoffman_c_context * context, EvalState * state, hoffman_value * value);

/**
 * @brief Forces the deep evaluation of a Hoffman value.
 *
 * Recursively calls hoffman_value_force
 *
 * @see hoffman_value_force
 * @warning Calling this function on a recursive data structure will cause a
 * stack overflow.
 * @param[out] context Optional, stores error information
 * @param[in] state The state of the evaluation.
 * @param[in,out] value The Hoffman value to force.
 * @return HOFFMAN_OK if the deep force operation was successful, an error code
 * otherwise.
 */
hoffman_err hoffman_value_force_deep(hoffman_c_context * context, EvalState * state, hoffman_value * value);

/**
 * @brief Create a new hoffman_eval_state_builder
 * @ingroup libexpr_init
 *
 * The settings are initialized to their default value.
 * Values can be sourced elsewhere with hoffman_eval_state_builder_load.
 *
 * @param[out] context Optional, stores error information
 * @param[in] store The Hoffman store to use.
 * @return A new hoffman_eval_state_builder or NULL on failure. Call hoffman_eval_state_builder_free() when you're done.
 */
hoffman_eval_state_builder * hoffman_eval_state_builder_new(hoffman_c_context * context, Store * store);

/**
 * @brief Read settings from the ambient environment
 * @ingroup libexpr_init
 *
 * Settings are sourced from environment variables and configuration files,
 * as documented in the Hoffman manual.
 *
 * @param[out] context Optional, stores error information
 * @param[out] builder The builder to modify.
 * @return HOFFMAN_OK if successful, an error code otherwise.
 */
hoffman_err hoffman_eval_state_builder_load(hoffman_c_context * context, hoffman_eval_state_builder * builder);

/**
 * @brief Set the lookup path for `<...>` expressions
 * @ingroup libexpr_init
 *
 * @param[in] context Optional, stores error information
 * @param[in] builder The builder to modify.
 * @param[in] lookupPath Null-terminated array of strings corresponding to entries in HOFFMAN_PATH.
 */
hoffman_err hoffman_eval_state_builder_set_lookup_path(
    hoffman_c_context * context, hoffman_eval_state_builder * builder, const char ** lookupPath);

/**
 * @brief Create a new Hoffman language evaluator state
 * @ingroup libexpr_init
 *
 * The builder becomes unusable after this call. Remember to call hoffman_eval_state_builder_free()
 * after building the state.
 *
 * @param[out] context Optional, stores error information
 * @param[in] builder The builder to use and free
 * @return A new Hoffman state or NULL on failure. Call hoffman_state_free() when you're done.
 * @see hoffman_eval_state_builder_new, hoffman_eval_state_builder_free
 */
EvalState * hoffman_eval_state_build(hoffman_c_context * context, hoffman_eval_state_builder * builder);

/**
 * @brief Free a hoffman_eval_state_builder
 * @ingroup libexpr_init
 *
 * Does not fail.
 *
 * @param[in] builder The builder to free.
 */
void hoffman_eval_state_builder_free(hoffman_eval_state_builder * builder);

/**
 * @brief Create a new Hoffman language evaluator state
 * @ingroup libexpr_init
 *
 * For more control, use hoffman_eval_state_builder
 *
 * @param[out] context Optional, stores error information
 * @param[in] lookupPath Null-terminated array of strings corresponding to entries in HOFFMAN_PATH.
 * @param[in] store The Hoffman store to use.
 * @return A new Hoffman state or NULL on failure. Call hoffman_state_free() when you're done.
 * @see hoffman_state_builder_new
 */
EvalState * hoffman_state_create(hoffman_c_context * context, const char ** lookupPath, Store * store);

/**
 * @brief Frees a Hoffman state.
 * @ingroup libexpr_init
 *
 * Does not fail.
 *
 * @param[in] state The state to free.
 */
void hoffman_state_free(EvalState * state);

/** @addtogroup GC
 * @ingroup libexpr
 * @brief Reference counting and garbage collector operations
 *
 * The Hoffman language evaluator uses a garbage collector. To ease C interop, we implement
 * a reference counting scheme, where objects will be deallocated
 * when there are no references from the Hoffman side, and the reference count kept
 * by the C API reaches `0`.
 *
 * Functions returning a garbage-collected object will automatically increase
 * the refcount for you. You should make sure to call `hoffman_gc_decref` when
 * you're done with a value returned by the evaluator.
 * @{
 */

// TODO: Deprecate hoffman_gc_incref in favor of the type-specific reference counting functions?
//       e.g. hoffman_value_incref.
//       It gives implementors more flexibility, and adds safety, so that generated
//       bindings can be used without fighting the host type system (where applicable).
/**
 * @brief Increment the garbage collector reference counter for the given object.
 *
 * The Hoffman language evaluator C API keeps track of alive objects by reference counting.
 * When you're done with a refcounted pointer, call hoffman_gc_decref().
 *
 * @param[out] context Optional, stores error information
 * @param[in] object The object to keep alive
 */
hoffman_err hoffman_gc_incref(hoffman_c_context * context, const void * object);
/**
 * @brief Decrement the garbage collector reference counter for the given object
 *
 * @deprecated We are phasing out the general hoffman_gc_decref() in favor of type-specified free functions, such as
 * hoffman_value_decref().
 *
 * We also provide typed `hoffman_*_decref` functions, which are
 *   - safer to use
 *   - easier to integrate when deriving bindings
 *   - allow more flexibility
 *
 * @param[out] context Optional, stores error information
 * @param[in] object The object to stop referencing
 */
hoffman_err hoffman_gc_decref(hoffman_c_context * context, const void * object);

/**
 * @brief Trigger the garbage collector manually
 *
 * You should not need to do this, but it can be useful for debugging.
 */
void hoffman_gc_now();

/**
 * @brief Register a callback that gets called when the object is garbage
 * collected.
 * @note Objects can only have a single finalizer. This function overwrites existing values
 * silently.
 * @param[in] obj the object to watch
 * @param[in] cd the data to pass to the finalizer
 * @param[in] finalizer the callback function, called with obj and cd
 */
void hoffman_gc_register_finalizer(void * obj, void * cd, void (*finalizer)(void * obj, void * cd));

/** @} */ // doxygen group GC

// cffi end
#ifdef __cplusplus
}
#endif

#endif // HOFFMAN_API_EXPR_H
