#pragma once
///@file

#include <rapidcheck/gen/Arbitrary.h>

#include "hoffman/util/hash.hh"

namespace rc {

using namespace hoffman;

template<>
struct Arbitrary<Hash>
{
    static Gen<Hash> arbitrary();
};

} // namespace rc
