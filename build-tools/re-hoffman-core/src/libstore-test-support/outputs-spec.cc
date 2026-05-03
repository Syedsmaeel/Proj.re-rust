#include "hoffman/store/tests/outputs-spec.hh"

#include <rapidcheck.h>

namespace rc {

Gen<hoffman::OutputsSpec> Arbitrary<hoffman::OutputsSpec>::arbitrary()
{
    using hoffman::OutputsSpec;
    using hoffman::StorePathName;
    using hoffman::StringSet;

    return gen::mapcat(
        gen::inRange<uint8_t>(0, std::variant_size_v<OutputsSpec::Raw>), [](uint8_t n) -> Gen<OutputsSpec> {
            switch (n) {
            case 0:
                return gen::just((OutputsSpec) OutputsSpec::All{});
            case 1:
                return gen::map(
                    gen::nonEmpty(
                        gen::container<StringSet>(
                            gen::map(gen::arbitrary<StorePathName>(), [](StorePathName n) { return n.name; }))),
                    [](StringSet names) { return (OutputsSpec) OutputsSpec::Names{names}; });
            default:
                assert(false);
            }
        });
}

} // namespace rc
