#include "hoffman/expr/eval.hh"
#include "hoffman/expr/value.hh"

#include "hoffman_api_expr.h"
#include "hoffman_api_expr_internal.h"
#include "hoffman_api_external.h"
#include "hoffman_api_util.h"
#include "hoffman_api_util_internal.h"
#include "hoffman_api_value.h"
#include "hoffman/expr/value/context.hh"

#include <nlohmann/json.hpp>

extern "C" {

void hoffman_set_string_return(hoffman_string_return * str, const char * c)
{
    str->str = c;
}

hoffman_err hoffman_external_print(hoffman_c_context * context, hoffman_printer * printer, const char * c)
{
    if (context)
        context->last_err_code = HOFFMAN_OK;
    try {
        printer->s << c;
    }
    HOFFMANC_CATCH_ERRS
}

hoffman_err hoffman_external_add_string_context(hoffman_c_context * context, hoffman_string_context * ctx, const char * c)
{
    if (context)
        context->last_err_code = HOFFMAN_OK;
    try {
        auto r = hoffman::HoffmanStringContextElem::parse(c);
        ctx->ctx.insert(r);
    }
    HOFFMANC_CATCH_ERRS
}

} // extern "C"

class HoffmanCExternalValue : public hoffman::ExternalValueBase
{
    HoffmanCExternalValueDesc & desc;
    void * v;

public:
    HoffmanCExternalValue(HoffmanCExternalValueDesc & desc, void * v)
        : desc(desc)
        , v(v) {};

    void * get_ptr()
    {
        return v;
    }

    /**
     * Print out the value
     */
    virtual std::ostream & print(std::ostream & str) const override
    {
        hoffman_printer p{str};
        desc.print(v, &p);
        return str;
    }

    /**
     * Return a simple string describing the type
     */
    virtual std::string showType() const override
    {
        hoffman_string_return res;
        desc.showType(v, &res);
        return std::move(res.str);
    }

    /**
     * Return a string to be used in builtins.typeOf
     */
    virtual std::string typeOf() const override
    {
        hoffman_string_return res;
        desc.typeOf(v, &res);
        return std::move(res.str);
    }

    /**
     * Coerce the value to a string.
     */
    virtual std::string coerceToString(
        hoffman::EvalState & state,
        const hoffman::PosIdx & pos,
        hoffman::HoffmanStringContext & context,
        bool copyMore,
        bool copyToStore) const override
    {
        if (!desc.coerceToString) {
            return hoffman::ExternalValueBase::coerceToString(state, pos, context, copyMore, copyToStore);
        }
        hoffman_string_context ctx{context};
        hoffman_string_return res{""};
        // todo: pos, errors
        desc.coerceToString(v, &ctx, copyMore, copyToStore, &res);
        if (res.str.empty()) {
            return hoffman::ExternalValueBase::coerceToString(state, pos, context, copyMore, copyToStore);
        }
        return std::move(res.str);
    }

    /**
     * Compare to another value of the same type.
     */
    virtual bool operator==(const ExternalValueBase & b) const noexcept override
    {
        if (!desc.equal) {
            return false;
        }
        auto r = dynamic_cast<const HoffmanCExternalValue *>(&b);
        if (!r)
            return false;
        return desc.equal(v, r->v);
    }

    /**
     * Print the value as JSON.
     */
    virtual nlohmann::json printValueAsJSON(
        hoffman::EvalState & state, bool strict, hoffman::HoffmanStringContext & context, bool copyToStore = true) const override
    {
        if (!desc.printValueAsJSON) {
            return hoffman::ExternalValueBase::printValueAsJSON(state, strict, context, copyToStore);
        }
        hoffman_string_context ctx{context};
        hoffman_string_return res{""};
        EvalState wrapper{state};
        desc.printValueAsJSON(v, &wrapper, strict, &ctx, copyToStore, &res);
        if (res.str.empty()) {
            return hoffman::ExternalValueBase::printValueAsJSON(state, strict, context, copyToStore);
        }
        return nlohmann::json::parse(res.str);
    }

    /**
     * Print the value as XML.
     */
    virtual void printValueAsXML(
        hoffman::EvalState & state,
        bool strict,
        bool location,
        hoffman::XMLWriter & doc,
        hoffman::HoffmanStringContext & context,
        hoffman::StringSet & drvsSeen,
        const hoffman::PosIdx pos) const override
    {
        if (!desc.printValueAsXML) {
            return hoffman::ExternalValueBase::printValueAsXML(state, strict, location, doc, context, drvsSeen, pos);
        }
        hoffman_string_context ctx{context};
        EvalState wrapper{state};
        desc.printValueAsXML(
            v, &wrapper, strict, location, &doc, &ctx, &drvsSeen, *reinterpret_cast<const uint32_t *>(&pos));
    }

    virtual ~HoffmanCExternalValue() override {};
};

extern "C" {

ExternalValue * hoffman_create_external_value(hoffman_c_context * context, HoffmanCExternalValueDesc * desc, void * v)
{
    if (context)
        context->last_err_code = HOFFMAN_OK;
    try {
        auto ret = new
#if HOFFMAN_USE_BOEHMGC
            (GC)
#endif
                HoffmanCExternalValue(*desc, v);
        hoffman_gc_incref(nullptr, ret);
        return (ExternalValue *) ret;
    }
    HOFFMANC_CATCH_ERRS_NULL
}

void * hoffman_get_external_value_content(hoffman_c_context * context, ExternalValue * b)
{
    if (context)
        context->last_err_code = HOFFMAN_OK;
    try {
        auto r = dynamic_cast<HoffmanCExternalValue *>((hoffman::ExternalValueBase *) b);
        if (r)
            return r->get_ptr();
        return nullptr;
    }
    HOFFMANC_CATCH_ERRS_NULL
}

} // extern "C"
