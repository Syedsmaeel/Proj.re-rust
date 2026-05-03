#pragma once
///@file

#include <rapidcheck/gen/Arbitrary.h>

#include "hoffman/store/path.hh"

namespace hoffman {

struct StorePathName
{
    std::string name;
};

// For rapidcheck
void showValue(const StorePath & p, std::ostream & os);

} // namespace hoffman

namespace rc {

template<>
struct Arbitrary<hoffman::StorePathName>
{
    static Gen<hoffman::StorePathName> arbitrary();
};

template<>
struct Arbitrary<hoffman::StorePath>
{
    static Gen<hoffman::StorePath> arbitrary();
};

} // namespace rc
