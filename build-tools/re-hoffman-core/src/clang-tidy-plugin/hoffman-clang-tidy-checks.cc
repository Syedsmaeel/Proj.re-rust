/**
 * @brief Custom clang-tidy checks for the Hoffman project.
 *
 * This module registers custom clang-tidy checks specific to the Hoffman codebase.
 * To add a new check:
 * 1. Create check-name.hh and check-name.cc in this directory
 * 2. Include the header here
 * 3. Register the check in addCheckFactories()
 * 4. Add the source file to meson.build
 * 5. Enable the check in .clang-tidy (e.g., hoffman-checkname)
 */

#include <clang-tidy/ClangTidyModule.h>
#include <clang-tidy/ClangTidyModuleRegistry.h>

namespace hoffman::clang_tidy {

using namespace clang;
using namespace clang::tidy;

class HoffmanClangTidyChecks : public ClangTidyModule
{
public:
    void addCheckFactories([[maybe_unused]] ClangTidyCheckFactories & CheckFactories) override
    {
        // Custom checks will be registered here.
        // Example:
        // CheckFactories.registerCheck<MyCustomCheck>("hoffman-my-custom-check");
    }
};

static ClangTidyModuleRegistry::Add<HoffmanClangTidyChecks> X("hoffman-module", "Adds Hoffman-specific checks");

} // namespace hoffman::clang_tidy
