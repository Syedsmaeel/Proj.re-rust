#pragma once
#include "hoffman/fetchers/fetch-settings.hh"
#include "hoffman/util/ref.hh"

/**
 * A shared reference to `hoffman::fetchers::Settings`
 * @see hoffman::fetchers::Settings
 */
struct hoffman_fetchers_settings
{
    hoffman::ref<hoffman::fetchers::Settings> settings;
};
