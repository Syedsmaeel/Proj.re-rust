#ifndef HOFFMAN_API_STORE_INTERNAL_H
#define HOFFMAN_API_STORE_INTERNAL_H
#include "hoffman/store/store-api.hh"
#include "hoffman/store/derivations.hh"

extern "C" {

struct Store
{
    hoffman::ref<hoffman::Store> ptr;
};

struct StorePath
{
    hoffman::StorePath path;
};

struct hoffman_derivation
{
    hoffman::Derivation drv;
};

} // extern "C"

#endif
