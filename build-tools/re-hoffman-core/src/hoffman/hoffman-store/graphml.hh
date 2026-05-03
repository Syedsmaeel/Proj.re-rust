#pragma once
///@file

#include "hoffman/store/store-api.hh"

namespace hoffman {

void printGraphML(ref<Store> store, StorePathSet && roots);

}
