#include <exception> // IWYU pragma: keep (Needed by rapidcheck on Darwin and FreeBSD)
#include <rapidcheck.h>

#include "hoffman/expr/tests/value/context.hh"

namespace rc {
using namespace hoffman;

Gen<HoffmanStringContextElem::DrvDeep> Arbitrary<HoffmanStringContextElem::DrvDeep>::arbitrary()
{
    return gen::map(gen::arbitrary<StorePath>(), [](StorePath drvPath) {
        return HoffmanStringContextElem::DrvDeep{
            .drvPath = drvPath,
        };
    });
}

Gen<HoffmanStringContextElem> Arbitrary<HoffmanStringContextElem>::arbitrary()
{
    return gen::mapcat(
        gen::inRange<uint8_t>(0, std::variant_size_v<HoffmanStringContextElem::Raw>),
        [](uint8_t n) -> Gen<HoffmanStringContextElem> {
            switch (n) {
            case 0:
                return gen::map(
                    gen::arbitrary<HoffmanStringContextElem::Opaque>(), [](HoffmanStringContextElem a) { return a; });
            case 1:
                return gen::map(
                    gen::arbitrary<HoffmanStringContextElem::DrvDeep>(), [](HoffmanStringContextElem a) { return a; });
            case 2:
                return gen::map(
                    gen::arbitrary<HoffmanStringContextElem::Built>(), [](HoffmanStringContextElem a) { return a; });
            default:
                assert(false);
            }
        });
}

} // namespace rc
