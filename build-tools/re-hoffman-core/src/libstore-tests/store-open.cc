#include <gtest/gtest.h>

#include "hoffman/store/store-open.hh"
#include "hoffman/store/store-reference.hh"
#include "hoffman/store/local-store.hh"
#include "hoffman/store/globals.hh"
#include "hoffman/util/file-system.hh"
#include "hoffman/util/finally.hh"

namespace hoffman {

TEST(StoreOpen, resolveStoreConfig_auto_default)
{
    // Save original settings
    //
    // TODO: resolveStoreConfig should not depend on global settings;
    // the test should not have to override them.
    auto originalStateDir = settings.hoffmanStateDir;
    Finally restoreStateDir([&]() { settings.hoffmanStateDir = originalStateDir; });

    // Set up a temporary writable state directory
    auto tmpDir = createTempDir();
    AutoDelete delTmpDir(tmpDir, true);
    auto stateDir = tmpDir / "var/hoffman";
    createDirs(stateDir);
    settings.hoffmanStateDir = stateDir;

    StoreReference ref{
        .variant = StoreReference::Auto{},
        .params = {},
    };

    auto config = resolveStoreConfig(std::move(ref));

    // With a writable state directory and no daemon socket, "auto" should resolve to LocalStore
    auto * localConfig = dynamic_cast<LocalStore::Config *>(config.get());
    ASSERT_NE(localConfig, nullptr);
    EXPECT_EQ(localConfig->getStateDir(), stateDir);
}

TEST(StoreOpen, resolveStoreConfig_auto_withParams)
{
    // Create a temporary directory with a writable state directory
    auto tmpDir = createTempDir();
    AutoDelete delTmpDir(tmpDir, true);
    auto stateDir = tmpDir / "var/hoffman";
    createDirs(stateDir);

    StoreReference ref{
        .variant = StoreReference::Auto{},
        .params = {{"state", stateDir.string()}},
    };

    auto config = resolveStoreConfig(std::move(ref));

    // With a writable state directory and no daemon socket, "auto" should resolve to LocalStore
    auto * localConfig = dynamic_cast<LocalStore::Config *>(config.get());
    ASSERT_NE(localConfig, nullptr);
    EXPECT_EQ(localConfig->getStateDir(), stateDir);
}

} // namespace hoffman
