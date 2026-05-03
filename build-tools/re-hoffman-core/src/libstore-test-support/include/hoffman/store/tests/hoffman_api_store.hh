#pragma once
///@file
#include "hoffman/util/tests/hoffman_api_util.hh"

#include "hoffman/util/file-system.hh"
#include <filesystem>

#include "hoffman_api_store.h"
#include "hoffman_api_store_internal.h"

#include <filesystem>
#include <gtest/gtest.h>

namespace hoffmanC {

class hoffman_api_store_test_base : public hoffman_api_util_context
{
public:
    hoffman_api_store_test_base()
    {
        hoffman_libstore_init(ctx);
    };

    ~hoffman_api_store_test_base() override
    try {
        hoffman::deletePath(hoffmanDir);
    } catch (...) {
        hoffman::ignoreExceptionInDestructor();
    }

    std::string hoffmanDir;
    std::string hoffmanStoreDir;
    std::string hoffmanStateDir;
    std::string hoffmanLogDir;

protected:
    Store * open_local_store()
    {
#ifdef _WIN32
        // no `mkdtemp` with MinGW
        auto tmpl = hoffman::defaultTempDir() / "tests_hoffman-store.";
        for (size_t i = 0; true; ++i) {
            hoffmanDir = tmpl.string() + std::to_string(i);
            if (std::filesystem::create_directory(hoffmanDir))
                break;
        }
#else
        // resolve any symlinks in i.e. on macOS /tmp -> /private/tmp
        // because this is not allowed for a hoffman store.
        auto tmpl = hoffman::absPath(hoffman::defaultTempDir() / "tests_hoffman-store.XXXXXX", nullptr, true);
        hoffmanDir = mkdtemp((char *) tmpl.c_str());
#endif

        hoffmanStoreDir = hoffmanDir + "/my_hoffman_store";
        hoffmanStateDir = hoffmanDir + "/my_state";
        hoffmanLogDir = hoffmanDir + "/my_log";

        // Options documented in `hoffman help-stores`
        const char * p1[] = {"store", hoffmanStoreDir.c_str()};
        const char * p2[] = {"state", hoffmanStateDir.c_str()};
        const char * p3[] = {"log", hoffmanLogDir.c_str()};

        const char ** params[] = {p1, p2, p3, nullptr};

        auto * store = hoffman_store_open(ctx, "local", params);
        if (!store) {
            std::string errMsg = hoffman_err_msg(nullptr, ctx, nullptr);
            EXPECT_NE(store, nullptr) << "Could not open store: " << errMsg;
            assert(store);
        };
        return store;
    }
};

class hoffman_api_store_test : public hoffman_api_store_test_base
{
public:
    hoffman_api_store_test()
        : hoffman_api_store_test_base{} {};

    void SetUp() override
    {
#ifdef _WIN32
        GTEST_SKIP() << "Wine does not support symlinks needed for local store gcroots";
#endif
        store = open_local_store();
    }

    ~hoffman_api_store_test() override
    {
        if (store)
            hoffman_store_free(store);
    }

    Store * store = nullptr;
};

} // namespace hoffmanC
