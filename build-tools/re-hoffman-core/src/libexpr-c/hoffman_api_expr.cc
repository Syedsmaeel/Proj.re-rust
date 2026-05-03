#include <cstring>
#include <stdexcept>
#include <string>

#include "hoffman/expr/eval.hh"
#include "hoffman/expr/eval-gc.hh"
#include "hoffman/store/globals.hh"
#include "hoffman/expr/eval-settings.hh"
#include "hoffman/util/ref.hh"

#include "hoffman_api_expr.h"
#include "hoffman_api_expr_internal.h"
#include "hoffman_api_store.h"
#include "hoffman_api_store_internal.h"
#include "hoffman_api_util.h"
#include "hoffman_api_util_internal.h"

#if HOFFMAN_USE_BOEHMGC
#  include <boost/unordered/concurrent_flat_map.hpp>
#endif

extern "C" {

hoffman_err hoffman_libexpr_init(hoffman_c_context * context)
{
    if (context)
        context->last_err_code = HOFFMAN_OK;
    {
        auto ret = hoffman_libutil_init(context);
        if (ret != HOFFMAN_OK)
            return ret;
    }
    {
        auto ret = hoffman_libstore_init(context);
        if (ret != HOFFMAN_OK)
            return ret;
    }
    try {
        hoffman::initGC();
    }
    HOFFMANC_CATCH_ERRS
}

hoffman_err hoffman_expr_eval_from_string(
    hoffman_c_context * context, EvalState * state, const char * expr, const char * path, hoffman_value * value)
{
    if (context)
        context->last_err_code = HOFFMAN_OK;
    try {
        hoffman::Expr * parsedExpr = state->state.parseExprFromString(expr, state->state.rootPath(hoffman::CanonPath(path)));
        state->state.eval(parsedExpr, *value->value);
        state->state.forceValue(*value->value, hoffman::noPos);
    }
    HOFFMANC_CATCH_ERRS
}

hoffman_err hoffman_value_call(hoffman_c_context * context, EvalState * state, Value * fn, hoffman_value * arg, hoffman_value * value)
{
    if (context)
        context->last_err_code = HOFFMAN_OK;
    try {
        state->state.callFunction(*fn->value, *arg->value, *value->value, hoffman::noPos);
        state->state.forceValue(*value->value, hoffman::noPos);
    }
    HOFFMANC_CATCH_ERRS
}

hoffman_err hoffman_value_call_multi(
    hoffman_c_context * context, EvalState * state, hoffman_value * fn, size_t nargs, hoffman_value ** args, hoffman_value * value)
{
    if (context)
        context->last_err_code = HOFFMAN_OK;

    std::vector<hoffman::Value *> internal_args;
    internal_args.reserve(nargs);
    for (size_t i = 0; i < nargs; i++)
        internal_args.push_back(args[i]->value);

    try {
        state->state.callFunction(*fn->value, {internal_args.data(), nargs}, *value->value, hoffman::noPos);
        state->state.forceValue(*value->value, hoffman::noPos);
    }
    HOFFMANC_CATCH_ERRS
}

hoffman_err hoffman_value_force(hoffman_c_context * context, EvalState * state, hoffman_value * value)
{
    if (context)
        context->last_err_code = HOFFMAN_OK;
    try {
        state->state.forceValue(*value->value, hoffman::noPos);
    }
    HOFFMANC_CATCH_ERRS
}

hoffman_err hoffman_value_force_deep(hoffman_c_context * context, EvalState * state, hoffman_value * value)
{
    if (context)
        context->last_err_code = HOFFMAN_OK;
    try {
        state->state.forceValueDeep(*value->value);
    }
    HOFFMANC_CATCH_ERRS
}

hoffman_eval_state_builder * hoffman_eval_state_builder_new(hoffman_c_context * context, Store * store)
{
    if (context)
        context->last_err_code = HOFFMAN_OK;
    try {
        auto readOnly = hoffman::make_ref<bool>(true);
        return new hoffman_eval_state_builder{
            .store = hoffman::ref<hoffman::Store>(store->ptr),
            .settings = hoffman::EvalSettings{/* &bool */ *readOnly},
            .fetchSettings = hoffman::fetchers::Settings{},
            .readOnlyMode = readOnly,
        };
    }
    HOFFMANC_CATCH_ERRS_NULL
}

void hoffman_eval_state_builder_free(hoffman_eval_state_builder * builder)
{
    delete builder;
}

hoffman_err hoffman_eval_state_builder_load(hoffman_c_context * context, hoffman_eval_state_builder * builder)
{
    if (context)
        context->last_err_code = HOFFMAN_OK;
    try {
        // TODO: load in one go?
        builder->settings.readOnlyMode = &hoffman::settings.readOnlyMode;
        loadConfFile(builder->settings);
        loadConfFile(builder->fetchSettings);
    }
    HOFFMANC_CATCH_ERRS
}

hoffman_err hoffman_eval_state_builder_set_lookup_path(
    hoffman_c_context * context, hoffman_eval_state_builder * builder, const char ** lookupPath_c)
{
    if (context)
        context->last_err_code = HOFFMAN_OK;
    try {
        hoffman::Strings lookupPath;
        if (lookupPath_c != nullptr)
            for (size_t i = 0; lookupPath_c[i] != nullptr; i++)
                lookupPath.push_back(lookupPath_c[i]);
        builder->lookupPath = hoffman::LookupPath::parse(lookupPath);
    }
    HOFFMANC_CATCH_ERRS
}

EvalState * hoffman_eval_state_build(hoffman_c_context * context, hoffman_eval_state_builder * builder)
{
    if (context)
        context->last_err_code = HOFFMAN_OK;
    try {
        auto fetchSettings = std::make_unique<hoffman::fetchers::Settings>(std::move(builder->fetchSettings));
        auto settings = std::make_unique<hoffman::EvalSettings>(std::move(builder->settings));
        auto ownedState =
            std::make_shared<hoffman::EvalState>(builder->lookupPath, builder->store, *fetchSettings, *settings);
        auto & stateRef = *ownedState;
        void * p = ::operator new(sizeof(EvalState), static_cast<std::align_val_t>(alignof(EvalState)));
        return new (p) EvalState{stateRef, std::move(fetchSettings), std::move(settings), std::move(ownedState)};
    }
    HOFFMANC_CATCH_ERRS_NULL
}

EvalState * hoffman_state_create(hoffman_c_context * context, const char ** lookupPath_c, Store * store)
{
    auto builder = hoffman_eval_state_builder_new(context, store);
    if (builder == nullptr)
        return nullptr;

    if (hoffman_eval_state_builder_load(context, builder) != HOFFMAN_OK)
        return nullptr;

    if (hoffman_eval_state_builder_set_lookup_path(context, builder, lookupPath_c) != HOFFMAN_OK)
        return nullptr;

    auto * state = hoffman_eval_state_build(context, builder);
    hoffman_eval_state_builder_free(builder);
    return state;
}

void hoffman_state_free(EvalState * state)
{
    if (state)
        state->~EvalState();
    operator delete(state, static_cast<std::align_val_t>(alignof(EvalState)));
}

#if HOFFMAN_USE_BOEHMGC
boost::concurrent_flat_map<
    const void *,
    unsigned int,
    std::hash<const void *>,
    std::equal_to<const void *>,
    traceable_allocator<std::pair<const void * const, unsigned int>>>
    hoffman_refcounts{};

hoffman_err hoffman_gc_incref(hoffman_c_context * context, const void * p)
{
    if (context)
        context->last_err_code = HOFFMAN_OK;
    try {
        hoffman_refcounts.insert_or_visit({p, 1}, [](auto & kv) { kv.second++; });
    }
    HOFFMANC_CATCH_ERRS
}

hoffman_err hoffman_gc_decref(hoffman_c_context * context, const void * p)
{

    if (context)
        context->last_err_code = HOFFMAN_OK;
    try {
        bool fail = true;
        hoffman_refcounts.erase_if(p, [&](auto & kv) {
            fail = false;
            return !--kv.second;
        });
        if (fail)
            throw std::runtime_error("hoffman_gc_decref: object was not referenced");
    }
    HOFFMANC_CATCH_ERRS
}

void hoffman_gc_now()
{
    GC_gcollect();
}

#else
hoffman_err hoffman_gc_incref(hoffman_c_context * context, const void *)
{
    if (context)
        context->last_err_code = HOFFMAN_OK;
    return HOFFMAN_OK;
}

hoffman_err hoffman_gc_decref(hoffman_c_context * context, const void *)
{
    if (context)
        context->last_err_code = HOFFMAN_OK;
    return HOFFMAN_OK;
}

void hoffman_gc_now() {}
#endif

hoffman_err hoffman_value_incref(hoffman_c_context * context, hoffman_value * x)
{
    return hoffman_gc_incref(context, (const void *) x);
}

hoffman_err hoffman_value_decref(hoffman_c_context * context, hoffman_value * x)
{
    return hoffman_gc_decref(context, (const void *) x);
}

void hoffman_gc_register_finalizer(void * obj, void * cd, void (*finalizer)(void * obj, void * cd))
{
#if HOFFMAN_USE_BOEHMGC
    GC_REGISTER_FINALIZER(obj, finalizer, cd, 0, 0);
#endif
}

} // extern "C"
