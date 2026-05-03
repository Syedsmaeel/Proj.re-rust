#ifndef HOFFMAN_API_EXPR_INTERNAL_H
#define HOFFMAN_API_EXPR_INTERNAL_H

#include <memory>

#include "hoffman/fetchers/fetch-settings.hh"
#include "hoffman/expr/eval.hh"
#include "hoffman/expr/eval-settings.hh"
#include "hoffman/expr/attr-set.hh"
#include "hoffman_api_value.h"
#include "hoffman/expr/search-path.hh"

extern "C" {

struct hoffman_eval_state_builder
{
    hoffman::ref<hoffman::Store> store;
    hoffman::EvalSettings settings;
    hoffman::fetchers::Settings fetchSettings;
    hoffman::LookupPath lookupPath;
    hoffman::ref<bool> readOnlyMode;
};

struct EvalState
{
    hoffman::EvalState & state;
    // Owned resources; null for temporary wrappers created in C API callbacks.
    std::unique_ptr<hoffman::fetchers::Settings> ownedFetchSettings;
    std::unique_ptr<hoffman::EvalSettings> ownedSettings;
    std::shared_ptr<hoffman::EvalState> ownedState;
};

struct BindingsBuilder
{
    hoffman::BindingsBuilder builder;
};

struct ListBuilder
{
    hoffman::ListBuilder builder;
};

struct hoffman_value
{
    hoffman::Value * value;
    /**
     * As we move to a managed heap, we need EvalMemory in more places. Ideally, we would take in EvalState or
     * EvalMemory as an argument when we need it, but we don't want to make changes to the stable C api, so we stuff it
     * into the hoffman_value that will get passed in to the relevant functions.
     */
    hoffman::EvalMemory * mem;
};

struct hoffman_string_return
{
    std::string str;
};

struct hoffman_printer
{
    std::ostream & s;
};

struct hoffman_string_context
{
    hoffman::HoffmanStringContext & ctx;
};

struct hoffman_realised_string
{
    std::string str;
    std::vector<StorePath> storePaths;
};

} // extern "C"

#endif // HOFFMAN_API_EXPR_INTERNAL_H
