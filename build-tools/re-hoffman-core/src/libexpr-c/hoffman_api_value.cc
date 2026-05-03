#include "hoffman/expr/attr-set.hh"
#include "hoffman/expr/eval-error.hh"
#include "hoffman/expr/eval.hh"
#include "hoffman/store/path.hh"
#include "hoffman/expr/primops.hh"
#include "hoffman/expr/value.hh"

#include "hoffman_api_expr.h"
#include "hoffman_api_expr_internal.h"
#include "hoffman_api_util.h"
#include "hoffman_api_util_internal.h"
#include "hoffman_api_store_internal.h"
#include "hoffman_api_value.h"

// Internal helper functions to check [in] and [out] `Value *` parameters
static const hoffman::Value & check_value_not_null(const hoffman_value * value)
{
    if (!value) {
        throw std::runtime_error("hoffman_value is null");
    }
    return *value->value;
}

static hoffman::Value & check_value_not_null(hoffman_value * value)
{
    if (!value) {
        throw std::runtime_error("hoffman_value is null");
    }
    return *value->value;
}

static const hoffman::Value & check_value_in(const hoffman_value * value)
{
    auto & v = check_value_not_null(value);
    if (!v.isValid()) {
        throw std::runtime_error("Uninitialized hoffman_value");
    }
    return v;
}

static hoffman::Value & check_value_in(hoffman_value * value)
{
    auto & v = check_value_not_null(value);
    if (!v.isValid()) {
        throw std::runtime_error("Uninitialized hoffman_value");
    }
    return v;
}

static hoffman::Value & check_value_out(hoffman_value * value)
{
    auto & v = check_value_not_null(value);
    if (v.isValid()) {
        throw std::runtime_error("hoffman_value already initialized. Variables are immutable");
    }
    return v;
}

static hoffman_value * new_hoffman_value(hoffman::Value * v, hoffman::EvalMemory & mem)
{
    hoffman_value * ret = new (mem.allocBytes(sizeof(hoffman_value))) hoffman_value{
        .value = v,
        .mem = &mem,
    };
    hoffman_gc_incref(nullptr, ret);
    return ret;
}

/**
 * Helper function to convert calls from hoffman into C API.
 *
 * Deals with errors and converts arguments from C++ into C types.
 */
static void hoffman_c_primop_wrapper(
    PrimOpFun f,
    void * userdata,
    int arity,
    hoffman::EvalState & state,
    const hoffman::PosIdx pos,
    hoffman::Value ** args,
    hoffman::Value & v)
{
    hoffman_c_context ctx;

    // v currently has a thunk, but the C API initializers require an uninitialized value.
    //
    // We can't destroy the thunk, because that makes it impossible to retry,
    // which is needed for tryEval and for evaluation drivers that evaluate more
    // than one value (e.g. an attrset with two derivations, both of which
    // reference v).
    //
    // Instead we create a temporary value, and then assign the result to v.
    // This does not give the primop definition access to the thunk, but that's
    // ok because we don't see a need for this yet (e.g. inspecting thunks,
    // or maybe something to make blackholes work better; we don't know).
    hoffman::Value vTmp;
    hoffman_value * vTmpPtr = new_hoffman_value(&vTmp, state.mem);

    std::vector<hoffman_value *> external_args;
    external_args.reserve(arity);
    for (int i = 0; i < arity; i++) {
        hoffman_value * external_arg = new_hoffman_value(args[i], state.mem);
        external_args.push_back(external_arg);
    }
    EvalState wrapper{state};
    f(userdata, &ctx, &wrapper, external_args.data(), vTmpPtr);

    if (ctx.last_err_code != HOFFMAN_OK) {
        if (ctx.last_err_code == HOFFMAN_ERR_RECOVERABLE) {
            state.error<hoffman::RecoverableEvalError>("Recoverable error from custom function: %s", *ctx.last_err)
                .atPos(pos)
                .debugThrow();
        } else {
            state.error<hoffman::EvalError>("Error from custom function: %s", *ctx.last_err).atPos(pos).debugThrow();
        }
    }

    if (!vTmp.isValid()) {
        state.error<hoffman::EvalError>("Implementation error in custom function: return value was not initialized")
            .atPos(pos)
            .debugThrow();
    }

    if (vTmp.type() == hoffman::nThunk) {
        // We might allow this in the future if it makes sense for the evaluator
        // e.g. implementing tail recursion by returning a thunk to the next
        // "iteration". Until then, this is most likely a mistake or misunderstanding.
        state.error<hoffman::EvalError>("Implementation error in custom function: return value must not be a thunk")
            .atPos(pos)
            .debugThrow();
    }

    v = vTmp;
}

extern "C" {

PrimOp * hoffman_alloc_primop(
    hoffman_c_context * context,
    PrimOpFun fun,
    int arity,
    const char * name,
    const char ** args,
    const char * doc,
    void * user_data)
{
    if (context)
        context->last_err_code = HOFFMAN_OK;
    try {
        using namespace std::placeholders;
        auto p = new
#if HOFFMAN_USE_BOEHMGC
            (GC)
#endif
                hoffman::PrimOp{
                    .name = name,
                    .args = {},
                    .arity = (size_t) arity,
                    .doc = doc,
                    .impl = std::bind(hoffman_c_primop_wrapper, fun, user_data, arity, _1, _2, _3, _4)};
        if (args)
            for (size_t i = 0; args[i]; i++)
                p->args.emplace_back(*args);
        hoffman_gc_incref(nullptr, p);
        return (PrimOp *) p;
    }
    HOFFMANC_CATCH_ERRS_NULL
}

hoffman_err hoffman_register_primop(hoffman_c_context * context, PrimOp * primOp)
{
    if (context)
        context->last_err_code = HOFFMAN_OK;
    try {
        hoffman::RegisterPrimOp r(std::move(*((hoffman::PrimOp *) primOp)));
    }
    HOFFMANC_CATCH_ERRS
}

hoffman_value * hoffman_alloc_value(hoffman_c_context * context, EvalState * state)
{
    if (context)
        context->last_err_code = HOFFMAN_OK;
    try {
        hoffman_value * res = new_hoffman_value(state->state.allocValue(), state->state.mem);
        return res;
    }
    HOFFMANC_CATCH_ERRS_NULL
}

ValueType hoffman_get_type(hoffman_c_context * context, const hoffman_value * value)
{
    if (context)
        context->last_err_code = HOFFMAN_OK;
    try {
        auto & v = check_value_in(value);
        using namespace hoffman;
        switch (v.type()) {
        case nThunk:
            return HOFFMAN_TYPE_THUNK;
        case nFailed:
            return HOFFMAN_TYPE_FAILED;
        case nInt:
            return HOFFMAN_TYPE_INT;
        case nFloat:
            return HOFFMAN_TYPE_FLOAT;
        case nBool:
            return HOFFMAN_TYPE_BOOL;
        case nString:
            return HOFFMAN_TYPE_STRING;
        case nPath:
            return HOFFMAN_TYPE_PATH;
        case nNull:
            return HOFFMAN_TYPE_NULL;
        case nAttrs:
            return HOFFMAN_TYPE_ATTRS;
        case nList:
            return HOFFMAN_TYPE_LIST;
        case nFunction:
            return HOFFMAN_TYPE_FUNCTION;
        case nExternal:
            return HOFFMAN_TYPE_EXTERNAL;
        }
        return HOFFMAN_TYPE_NULL;
    }
    HOFFMANC_CATCH_ERRS_RES(HOFFMAN_TYPE_NULL);
}

const char * hoffman_get_typename(hoffman_c_context * context, const hoffman_value * value)
{
    if (context)
        context->last_err_code = HOFFMAN_OK;
    try {
        auto & v = check_value_in(value);
        auto s = hoffman::showType(v);
        return strdup(s.c_str());
    }
    HOFFMANC_CATCH_ERRS_NULL
}

bool hoffman_get_bool(hoffman_c_context * context, const hoffman_value * value)
{
    if (context)
        context->last_err_code = HOFFMAN_OK;
    try {
        auto & v = check_value_in(value);
        assert(v.type() == hoffman::nBool);
        return v.boolean();
    }
    HOFFMANC_CATCH_ERRS_RES(false);
}

hoffman_err
hoffman_get_string(hoffman_c_context * context, const hoffman_value * value, hoffman_get_string_callback callback, void * user_data)
{
    if (context)
        context->last_err_code = HOFFMAN_OK;
    try {
        auto & v = check_value_in(value);
        assert(v.type() == hoffman::nString);
        call_hoffman_get_string_callback(v.string_view(), callback, user_data);
    }
    HOFFMANC_CATCH_ERRS
}

const char * hoffman_get_path_string(hoffman_c_context * context, const hoffman_value * value)
{
    if (context)
        context->last_err_code = HOFFMAN_OK;
    try {
        auto & v = check_value_in(value);
        assert(v.type() == hoffman::nPath);
        // NOTE (from @yorickvP)
        // v._path.path should work but may not be how Eelco intended it.
        // Long-term this function should be rewritten to copy some data into a
        // user-allocated string.
        // We could use v.path().to_string().c_str(), but I'm concerned this
        // crashes. Looks like .path() allocates a CanonPath with a copy of the
        // string, then it gets the underlying data from that.
        return v.pathStr();
    }
    HOFFMANC_CATCH_ERRS_NULL
}

unsigned int hoffman_get_list_size(hoffman_c_context * context, const hoffman_value * value)
{
    if (context)
        context->last_err_code = HOFFMAN_OK;
    try {
        auto & v = check_value_in(value);
        assert(v.type() == hoffman::nList);
        return v.listSize();
    }
    HOFFMANC_CATCH_ERRS_RES(0);
}

unsigned int hoffman_get_attrs_size(hoffman_c_context * context, const hoffman_value * value)
{
    if (context)
        context->last_err_code = HOFFMAN_OK;
    try {
        auto & v = check_value_in(value);
        assert(v.type() == hoffman::nAttrs);
        return v.attrs()->size();
    }
    HOFFMANC_CATCH_ERRS_RES(0);
}

double hoffman_get_float(hoffman_c_context * context, const hoffman_value * value)
{
    if (context)
        context->last_err_code = HOFFMAN_OK;
    try {
        auto & v = check_value_in(value);
        assert(v.type() == hoffman::nFloat);
        return v.fpoint();
    }
    HOFFMANC_CATCH_ERRS_RES(0.0);
}

int64_t hoffman_get_int(hoffman_c_context * context, const hoffman_value * value)
{
    if (context)
        context->last_err_code = HOFFMAN_OK;
    try {
        auto & v = check_value_in(value);
        assert(v.type() == hoffman::nInt);
        return v.integer().value;
    }
    HOFFMANC_CATCH_ERRS_RES(0);
}

ExternalValue * hoffman_get_external(hoffman_c_context * context, hoffman_value * value)
{
    if (context)
        context->last_err_code = HOFFMAN_OK;
    try {
        auto & v = check_value_out(value);
        assert(v.type() == hoffman::nExternal);
        return (ExternalValue *) v.external();
    }
    HOFFMANC_CATCH_ERRS_NULL;
}

hoffman_value * hoffman_get_list_byidx(hoffman_c_context * context, const hoffman_value * value, EvalState * state, unsigned int ix)
{
    if (context)
        context->last_err_code = HOFFMAN_OK;
    try {
        auto & v = check_value_in(value);
        assert(v.type() == hoffman::nList);
        if (ix >= v.listSize()) {
            hoffman_set_err_msg(context, HOFFMAN_ERR_KEY, "list index out of bounds");
            return nullptr;
        }
        auto * p = v.listView()[ix];
        if (p == nullptr)
            return nullptr;
        state->state.forceValue(*p, hoffman::noPos);
        return new_hoffman_value(p, state->state.mem);
    }
    HOFFMANC_CATCH_ERRS_NULL
}

hoffman_value *
hoffman_get_list_byidx_lazy(hoffman_c_context * context, const hoffman_value * value, EvalState * state, unsigned int ix)
{
    if (context)
        context->last_err_code = HOFFMAN_OK;
    try {
        auto & v = check_value_in(value);
        assert(v.type() == hoffman::nList);
        if (ix >= v.listSize()) {
            hoffman_set_err_msg(context, HOFFMAN_ERR_KEY, "list index out of bounds");
            return nullptr;
        }
        auto * p = v.listView()[ix];
        // Note: intentionally NOT calling forceValue() to keep the element lazy
        return new_hoffman_value(p, state->state.mem);
    }
    HOFFMANC_CATCH_ERRS_NULL
}

hoffman_value * hoffman_get_attr_byname(hoffman_c_context * context, const hoffman_value * value, EvalState * state, const char * name)
{
    if (context)
        context->last_err_code = HOFFMAN_OK;
    try {
        auto & v = check_value_in(value);
        assert(v.type() == hoffman::nAttrs);
        hoffman::Symbol s = state->state.symbols.create(name);
        auto attr = v.attrs()->get(s);
        if (attr) {
            state->state.forceValue(*attr->value, hoffman::noPos);
            return new_hoffman_value(attr->value, state->state.mem);
        }
        hoffman_set_err_msg(context, HOFFMAN_ERR_KEY, "missing attribute");
        return nullptr;
    }
    HOFFMANC_CATCH_ERRS_NULL
}

hoffman_value *
hoffman_get_attr_byname_lazy(hoffman_c_context * context, const hoffman_value * value, EvalState * state, const char * name)
{
    if (context)
        context->last_err_code = HOFFMAN_OK;
    try {
        auto & v = check_value_in(value);
        assert(v.type() == hoffman::nAttrs);
        hoffman::Symbol s = state->state.symbols.create(name);
        auto attr = v.attrs()->get(s);
        if (attr) {
            // Note: intentionally NOT calling forceValue() to keep the attribute lazy
            return new_hoffman_value(attr->value, state->state.mem);
        }
        hoffman_set_err_msg(context, HOFFMAN_ERR_KEY, "missing attribute");
        return nullptr;
    }
    HOFFMANC_CATCH_ERRS_NULL
}

bool hoffman_has_attr_byname(hoffman_c_context * context, const hoffman_value * value, EvalState * state, const char * name)
{
    if (context)
        context->last_err_code = HOFFMAN_OK;
    try {
        auto & v = check_value_in(value);
        assert(v.type() == hoffman::nAttrs);
        hoffman::Symbol s = state->state.symbols.create(name);
        auto attr = v.attrs()->get(s);
        if (attr)
            return true;
        return false;
    }
    HOFFMANC_CATCH_ERRS_RES(false);
}

static void collapse_attrset_layer_chain_if_needed(hoffman::Value & v, EvalState * state)
{
    auto & attrs = *v.attrs();
    if (attrs.isLayered()) {
        auto bindings = state->state.buildBindings(attrs.size());
        std::ranges::copy(attrs, std::back_inserter(bindings));
        v.mkAttrs(bindings);
    }
}

hoffman_value *
hoffman_get_attr_byidx(hoffman_c_context * context, hoffman_value * value, EvalState * state, unsigned int i, const char ** name)
{
    if (context)
        context->last_err_code = HOFFMAN_OK;
    try {
        auto & v = check_value_in(value);
        collapse_attrset_layer_chain_if_needed(v, state);
        if (i >= v.attrs()->size()) {
            hoffman_set_err_msg(context, HOFFMAN_ERR_KEY, "attribute index out of bounds");
            return nullptr;
        }
        const hoffman::Attr & a = (*v.attrs())[i];
        *name = state->state.symbols[a.name].c_str();
        state->state.forceValue(*a.value, hoffman::noPos);
        return new_hoffman_value(a.value, state->state.mem);
    }
    HOFFMANC_CATCH_ERRS_NULL
}

hoffman_value * hoffman_get_attr_byidx_lazy(
    hoffman_c_context * context, hoffman_value * value, EvalState * state, unsigned int i, const char ** name)
{
    if (context)
        context->last_err_code = HOFFMAN_OK;
    try {
        auto & v = check_value_in(value);
        collapse_attrset_layer_chain_if_needed(v, state);
        if (i >= v.attrs()->size()) {
            hoffman_set_err_msg(context, HOFFMAN_ERR_KEY, "attribute index out of bounds (Hoffman C API contract violation)");
            return nullptr;
        }
        const hoffman::Attr & a = (*v.attrs())[i];
        *name = state->state.symbols[a.name].c_str();
        // Note: intentionally NOT calling forceValue() to keep the attribute lazy
        return new_hoffman_value(a.value, state->state.mem);
    }
    HOFFMANC_CATCH_ERRS_NULL
}

const char * hoffman_get_attr_name_byidx(hoffman_c_context * context, hoffman_value * value, EvalState * state, unsigned int i)
{
    if (context)
        context->last_err_code = HOFFMAN_OK;
    try {
        auto & v = check_value_in(value);
        collapse_attrset_layer_chain_if_needed(v, state);
        if (i >= v.attrs()->size()) {
            hoffman_set_err_msg(context, HOFFMAN_ERR_KEY, "attribute index out of bounds (Hoffman C API contract violation)");
            return nullptr;
        }
        const hoffman::Attr & a = (*v.attrs())[i];
        return state->state.symbols[a.name].c_str();
    }
    HOFFMANC_CATCH_ERRS_NULL
}

hoffman_err hoffman_init_bool(hoffman_c_context * context, hoffman_value * value, bool b)
{
    if (context)
        context->last_err_code = HOFFMAN_OK;
    try {
        auto & v = check_value_out(value);
        v.mkBool(b);
    }
    HOFFMANC_CATCH_ERRS
}

// todo string context
hoffman_err hoffman_init_string(hoffman_c_context * context, hoffman_value * value, const char * str)
{
    if (context)
        context->last_err_code = HOFFMAN_OK;
    try {
        auto & v = check_value_out(value);
        v.mkString(std::string_view(str), *value->mem);
    }
    HOFFMANC_CATCH_ERRS
}

hoffman_err hoffman_init_path_string(hoffman_c_context * context, EvalState * s, hoffman_value * value, const char * str)
{
    if (context)
        context->last_err_code = HOFFMAN_OK;
    try {
        auto & v = check_value_out(value);
        v.mkPath(s->state.rootPath(hoffman::CanonPath(str)), s->state.mem);
    }
    HOFFMANC_CATCH_ERRS
}

hoffman_err hoffman_init_float(hoffman_c_context * context, hoffman_value * value, double d)
{
    if (context)
        context->last_err_code = HOFFMAN_OK;
    try {
        auto & v = check_value_out(value);
        v.mkFloat(d);
    }
    HOFFMANC_CATCH_ERRS
}

hoffman_err hoffman_init_int(hoffman_c_context * context, hoffman_value * value, int64_t i)
{
    if (context)
        context->last_err_code = HOFFMAN_OK;
    try {
        auto & v = check_value_out(value);
        v.mkInt(i);
    }
    HOFFMANC_CATCH_ERRS
}

hoffman_err hoffman_init_null(hoffman_c_context * context, hoffman_value * value)
{
    if (context)
        context->last_err_code = HOFFMAN_OK;
    try {
        auto & v = check_value_out(value);
        v.mkNull();
    }
    HOFFMANC_CATCH_ERRS
}

hoffman_err hoffman_init_apply(hoffman_c_context * context, hoffman_value * value, hoffman_value * fn, hoffman_value * arg)
{
    if (context)
        context->last_err_code = HOFFMAN_OK;
    try {
        auto & v = check_value_not_null(value);
        auto & f = check_value_not_null(fn);
        auto & a = check_value_not_null(arg);
        v.mkApp(&f, &a);
    }
    HOFFMANC_CATCH_ERRS
}

hoffman_err hoffman_init_external(hoffman_c_context * context, hoffman_value * value, ExternalValue * val)
{
    if (context)
        context->last_err_code = HOFFMAN_OK;
    try {
        auto & v = check_value_out(value);
        auto r = (hoffman::ExternalValueBase *) val;
        v.mkExternal(r);
    }
    HOFFMANC_CATCH_ERRS
}

ListBuilder * hoffman_make_list_builder(hoffman_c_context * context, EvalState * state, size_t capacity)
{
    if (context)
        context->last_err_code = HOFFMAN_OK;
    try {
        auto builder = state->state.buildList(capacity);
        return new
#if HOFFMAN_USE_BOEHMGC
            (NoGC)
#endif
                ListBuilder{std::move(builder)};
    }
    HOFFMANC_CATCH_ERRS_NULL
}

hoffman_err
hoffman_list_builder_insert(hoffman_c_context * context, ListBuilder * list_builder, unsigned int index, hoffman_value * value)
{
    if (context)
        context->last_err_code = HOFFMAN_OK;
    try {
        auto & e = check_value_not_null(value);
        list_builder->builder[index] = &e;
    }
    HOFFMANC_CATCH_ERRS
}

void hoffman_list_builder_free(ListBuilder * list_builder)
{
#if HOFFMAN_USE_BOEHMGC
    GC_FREE(list_builder);
#else
    delete list_builder;
#endif
}

hoffman_err hoffman_make_list(hoffman_c_context * context, ListBuilder * list_builder, hoffman_value * value)
{
    if (context)
        context->last_err_code = HOFFMAN_OK;
    try {
        auto & v = check_value_out(value);
        v.mkList(list_builder->builder);
    }
    HOFFMANC_CATCH_ERRS
}

hoffman_err hoffman_init_primop(hoffman_c_context * context, hoffman_value * value, PrimOp * p)
{
    if (context)
        context->last_err_code = HOFFMAN_OK;
    try {
        auto & v = check_value_out(value);
        v.mkPrimOp((hoffman::PrimOp *) p);
    }
    HOFFMANC_CATCH_ERRS
}

hoffman_err hoffman_copy_value(hoffman_c_context * context, hoffman_value * value, const hoffman_value * source)
{
    if (context)
        context->last_err_code = HOFFMAN_OK;
    try {
        auto & v = check_value_out(value);
        auto & s = check_value_in(source);
        v = s;
    }
    HOFFMANC_CATCH_ERRS
}

hoffman_err hoffman_make_attrs(hoffman_c_context * context, hoffman_value * value, BindingsBuilder * b)
{
    if (context)
        context->last_err_code = HOFFMAN_OK;
    try {
        auto & v = check_value_out(value);
        v.mkAttrs(b->builder);
    }
    HOFFMANC_CATCH_ERRS
}

BindingsBuilder * hoffman_make_bindings_builder(hoffman_c_context * context, EvalState * state, size_t capacity)
{
    if (context)
        context->last_err_code = HOFFMAN_OK;
    try {
        auto bb = state->state.buildBindings(capacity);
        return new
#if HOFFMAN_USE_BOEHMGC
            (NoGC)
#endif
                BindingsBuilder{std::move(bb)};
    }
    HOFFMANC_CATCH_ERRS_NULL
}

hoffman_err hoffman_bindings_builder_insert(hoffman_c_context * context, BindingsBuilder * bb, const char * name, hoffman_value * value)
{
    if (context)
        context->last_err_code = HOFFMAN_OK;
    try {
        auto & v = check_value_not_null(value);
        hoffman::Symbol s = bb->builder.symbols.get().create(name);
        bb->builder.insert(s, &v);
    }
    HOFFMANC_CATCH_ERRS
}

void hoffman_bindings_builder_free(BindingsBuilder * bb)
{
#if HOFFMAN_USE_BOEHMGC
    GC_FREE((hoffman::BindingsBuilder *) bb);
#else
    delete (hoffman::BindingsBuilder *) bb;
#endif
}

hoffman_realised_string * hoffman_string_realise(hoffman_c_context * context, EvalState * state, hoffman_value * value, bool isIFD)
{
    if (context)
        context->last_err_code = HOFFMAN_OK;
    try {
        auto & v = check_value_in(value);
        hoffman::StorePathSet storePaths;
        auto s = state->state.realiseString(v, &storePaths, isIFD);

        // Convert to the C API StorePath type and convert to vector for index-based access
        std::vector<StorePath> vec;
        for (auto & sp : storePaths) {
            vec.push_back(StorePath{sp});
        }

        return new hoffman_realised_string{.str = s, .storePaths = vec};
    }
    HOFFMANC_CATCH_ERRS_NULL
}

void hoffman_realised_string_free(hoffman_realised_string * s)
{
    delete s;
}

size_t hoffman_realised_string_get_buffer_size(hoffman_realised_string * s)
{
    return s->str.size();
}

const char * hoffman_realised_string_get_buffer_start(hoffman_realised_string * s)
{
    return s->str.data();
}

size_t hoffman_realised_string_get_store_path_count(hoffman_realised_string * s)
{
    return s->storePaths.size();
}

const StorePath * hoffman_realised_string_get_store_path(hoffman_realised_string * s, size_t i)
{
    return &s->storePaths[i];
}

} // extern "C"
