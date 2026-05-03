#pragma once
///@file

#include <rapidcheck/gen/Arbitrary.h>

#include "hoffman/expr/value/context.hh"
#include "hoffman/store/tests/derived-path.hh" // IWYU pragma: keep

namespace rc {

template<>
struct Arbitrary<hoffman::HoffmanStringContextElem::DrvDeep>
{
    static Gen<hoffman::HoffmanStringContextElem::DrvDeep> arbitrary();
};

template<>
struct Arbitrary<hoffman::HoffmanStringContextElem>
{
    static Gen<hoffman::HoffmanStringContextElem> arbitrary();
};

} // namespace rc
