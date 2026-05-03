#include <exception> // IWYU pragma: keep (Needed by rapidcheck on Darwin and FreeBSD)
#include <rapidcheck.h>

#include "hoffman/util/hash.hh"

#include "hoffman/util/tests/hash.hh"

namespace rc {

using namespace hoffman;

Gen<Hash> Arbitrary<Hash>::arbitrary()
{
    Hash prototype(HashAlgorithm::SHA1);
    return gen::apply(
        [](const std::vector<uint8_t> & v) {
            Hash hash(HashAlgorithm::SHA1);
            assert(v.size() == hash.hashSize);
            std::copy(v.begin(), v.end(), hash.hash);
            return hash;
        },
        gen::container<std::vector<uint8_t>>(prototype.hashSize, gen::arbitrary<uint8_t>()));
}

} // namespace rc
