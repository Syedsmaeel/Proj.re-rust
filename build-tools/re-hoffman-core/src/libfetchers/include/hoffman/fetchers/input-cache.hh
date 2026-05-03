#pragma once

#include "hoffman/fetchers/fetchers.hh"

namespace hoffman::fetchers {

enum class UseRegistries : int;
struct Settings;

struct InputCache
{
    struct CachedResult
    {
        ref<SourceAccessor> accessor;
        Input resolvedInput;
        Input lockedInput;
        Attrs extraAttrs;
    };

    CachedResult
    getAccessor(const Settings & settings, Store & store, const Input & originalInput, UseRegistries useRegistries);

    struct CachedInput
    {
        Input lockedInput;
        ref<SourceAccessor> accessor;
        Attrs extraAttrs;
    };

    virtual std::optional<CachedInput> lookup(const Input & originalInput) const = 0;

    virtual void upsert(Input key, CachedInput cachedInput) = 0;

    virtual void clear() = 0;

    static ref<InputCache> create();

    virtual ~InputCache() = default;
};

} // namespace hoffman::fetchers
