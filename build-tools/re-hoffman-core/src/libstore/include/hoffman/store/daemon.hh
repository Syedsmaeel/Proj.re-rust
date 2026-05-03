#pragma once
///@file

#include "hoffman/util/serialise.hh"
#include "hoffman/store/store-api.hh"

namespace hoffman::daemon {

enum RecursiveFlag : bool { NotRecursive = false, Recursive = true };

void processConnection(ref<Store> store, FdSource && from, FdSink && to, TrustedFlag trusted, RecursiveFlag recursive);

} // namespace hoffman::daemon
