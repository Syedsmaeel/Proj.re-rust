#pragma once
///@file

#include <exception> // IWYU pragma: keep (Needed by rapidcheck on Darwin and FreeBSD)
#include <rapidcheck/gen/Arbitrary.h>

#include "hoffman/store/outputs-spec.hh"

#include "hoffman/store/tests/path.hh"

namespace rc {

template<>
struct Arbitrary<hoffman::OutputsSpec>
{
    static Gen<hoffman::OutputsSpec> arbitrary();
};

} // namespace rc
