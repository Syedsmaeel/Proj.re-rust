#pragma once
///@file

#include <rapidcheck/gen/Arbitrary.h>

#include "hoffman/store/derived-path.hh"

#include "hoffman/store/tests/path.hh"
#include "hoffman/store/tests/outputs-spec.hh"

namespace rc {

template<>
struct Arbitrary<hoffman::SingleDerivedPath::Opaque>
{
    static Gen<hoffman::SingleDerivedPath::Opaque> arbitrary();
};

template<>
struct Arbitrary<hoffman::SingleDerivedPath::Built>
{
    static Gen<hoffman::SingleDerivedPath::Built> arbitrary();
};

template<>
struct Arbitrary<hoffman::SingleDerivedPath>
{
    static Gen<hoffman::SingleDerivedPath> arbitrary();
};

template<>
struct Arbitrary<hoffman::DerivedPath::Built>
{
    static Gen<hoffman::DerivedPath::Built> arbitrary();
};

template<>
struct Arbitrary<hoffman::DerivedPath>
{
    static Gen<hoffman::DerivedPath> arbitrary();
};

} // namespace rc
