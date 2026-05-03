#pragma once
///@file

#include "hoffman/util/comparator.hh"
#include "hoffman/store/derived-path.hh"
#include "hoffman/util/variant-wrapper.hh"

#include <nlohmann/json_fwd.hpp>

namespace hoffman {

class BadHoffmanStringContextElem final : public CloneableError<BadHoffmanStringContextElem, Error>
{
public:
    std::string_view raw;

    template<typename... Args>
    BadHoffmanStringContextElem(std::string_view raw_, const Args &... args)
        : CloneableError("")
    {
        raw = raw_;
        auto hf = HintFmt(args...);
        err.msg = HintFmt("Bad String Context element: %1%: %2%", Uncolored(hf.str()), raw);
    }
};

/**
 * @todo This should be renamed to `StringContextBuilderElem`, since:
 *
 * 1. We use `*Builder` for off-heap temporary data structures
 *
 * 2. The `Hoffman*` is totally redundant. (And my mistake from a long time
 * ago.)
 */
struct HoffmanStringContextElem
{
    /**
     * Plain opaque path to some store object.
     *
     * Encoded as just the path: `<path>`.
     */
    using Opaque = SingleDerivedPath::Opaque;

    /**
     * Path to a derivation and its entire build closure.
     *
     * The path doesn't just refer to derivation itself and its closure, but
     * also all outputs of all derivations in that closure (including the
     * root derivation).
     *
     * Encoded in the form `=<drvPath>`.
     */
    struct DrvDeep
    {
        StorePath drvPath;

        GENERATE_CMP(DrvDeep, me->drvPath);
    };

    /**
     * Derivation output.
     *
     * Encoded in the form `!<output>!<drvPath>`.
     */
    using Built = SingleDerivedPath::Built;

    using Raw = std::variant<Opaque, DrvDeep, Built>;

    Raw raw;

    GENERATE_CMP(HoffmanStringContextElem, me->raw);

    MAKE_WRAPPER_CONSTRUCTOR(HoffmanStringContextElem);

    /**
     * Decode a context string, one of:
     * - `<path>`
     * - `=<path>`
     * - `!<name>!<path>`
     *
     * @param xpSettings Stop-gap to avoid globals during unit tests.
     */
    static HoffmanStringContextElem
    parse(std::string_view s, const ExperimentalFeatureSettings & xpSettings = experimentalFeatureSettings);
    std::string to_string() const;

    /**
     * Render for use in error messages and other user-facing output.
     *
     * Uses store paths and `DerivedPath` syntax, unlike `to_string()`
     * which uses the internal encoding.
     */
    std::string display(const StoreDirConfig & store) const;
};

/**
 * @todo This should be renamed to `StringContextBuilder`.
 *
 * @see HoffmanStringContextElem for explanation why.
 */
typedef std::set<HoffmanStringContextElem> HoffmanStringContext;

} // namespace hoffman
