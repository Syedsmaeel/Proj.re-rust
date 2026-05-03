#pragma once
///@file

#include "hoffman/store/store-api.hh"

namespace hoffman {

void printDotGraph(ref<Store> store, StorePathSet && roots);

}
