#include "hoffman_api_fetchers.h"
#include "hoffman_api_fetchers_internal.hh"
#include "hoffman_api_util_internal.h"

extern "C" {

hoffman_fetchers_settings * hoffman_fetchers_settings_new(hoffman_c_context * context)
{
    try {
        auto fetchersSettings = hoffman::make_ref<hoffman::fetchers::Settings>(hoffman::fetchers::Settings{});
        return new hoffman_fetchers_settings{
            .settings = fetchersSettings,
        };
    }
    HOFFMANC_CATCH_ERRS_NULL
}

void hoffman_fetchers_settings_free(hoffman_fetchers_settings * settings)
{
    delete settings;
}

} // extern "C"
