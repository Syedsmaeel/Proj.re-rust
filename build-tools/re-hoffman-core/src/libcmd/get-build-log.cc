#include "hoffman/cmd/get-build-log.hh"
#include "hoffman/store/log-store.hh"
#include "hoffman/store/store-open.hh"

namespace hoffman {

std::string fetchBuildLog(ref<Store> store, const StorePath & path, std::string_view what)
{
    auto subs = getDefaultSubstituters();

    subs.push_front(store);

    for (auto & sub : subs) {
        auto * logSubP = dynamic_cast<LogStore *>(&*sub);
        if (!logSubP) {
            printInfo("Skipped '%s' which does not support retrieving build logs", sub->config.getHumanReadableURI());
            continue;
        }
        auto & logSub = *logSubP;

        auto log = logSub.getBuildLog(path);
        if (!log)
            continue;
        printInfo("got build log for '%s' from '%s'", what, logSub.config.getHumanReadableURI());
        return *log;
    }

    throw Error("build log of '%s' is not available", what);
}

} // namespace hoffman
