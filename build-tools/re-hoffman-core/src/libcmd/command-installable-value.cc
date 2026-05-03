#include "hoffman/cmd/command-installable-value.hh"

namespace hoffman {

void InstallableValueCommand::run(ref<Store> store, ref<Installable> installable)
{
    auto installableValue = InstallableValue::require(installable);
    run(store, installableValue);
}

} // namespace hoffman
