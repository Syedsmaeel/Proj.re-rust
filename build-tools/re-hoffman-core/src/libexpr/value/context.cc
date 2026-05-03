#include "hoffman/util/util.hh"
#include "hoffman/expr/value/context.hh"
#include "hoffman/store/store-dir-config.hh"

namespace hoffman {

HoffmanStringContextElem HoffmanStringContextElem::parse(std::string_view s0, const ExperimentalFeatureSettings & xpSettings)
{
    std::string_view s = s0;

    auto parseRest = [&](this auto & parseRest) -> SingleDerivedPath {
        // Case on whether there is a '!'
        size_t index = s.find("!");
        if (index == std::string_view::npos) {
            return SingleDerivedPath::Opaque{
                .path = StorePath{s},
            };
        } else {
            std::string output{s.substr(0, index)};
            // Advance string to parse after the '!'
            s = s.substr(index + 1);
            auto drv = make_ref<SingleDerivedPath>(parseRest());
            drvRequireExperiment(*drv, xpSettings);
            return SingleDerivedPath::Built{
                .drvPath = std::move(drv),
                .output = std::move(output),
            };
        }
    };

    if (s.size() == 0) {
        throw BadHoffmanStringContextElem(s0, "String context element should never be an empty string");
    }

    switch (s.at(0)) {
    case '!': {
        // Advance string to parse after the '!'
        s = s.substr(1);

        // Find *second* '!'
        if (s.find("!") == std::string_view::npos) {
            throw BadHoffmanStringContextElem(s0, "String content element beginning with '!' should have a second '!'");
        }

        return std::visit([&](auto x) -> HoffmanStringContextElem { return std::move(x); }, parseRest());
    }
    case '=': {
        return HoffmanStringContextElem::DrvDeep{
            .drvPath = StorePath{s.substr(1)},
        };
    }
    default: {
        // Ensure no '!'
        if (s.find("!") != std::string_view::npos) {
            throw BadHoffmanStringContextElem(
                s0, "String content element not beginning with '!' should not have a second '!'");
        }
        return std::visit([&](auto x) -> HoffmanStringContextElem { return std::move(x); }, parseRest());
    }
    }
}

std::string HoffmanStringContextElem::to_string() const
{
    std::string res;

    std::function<void(const SingleDerivedPath &)> toStringRest;
    toStringRest = [&](auto & p) {
        std::visit(
            overloaded{
                [&](const SingleDerivedPath::Opaque & o) { res += o.path.to_string(); },
                [&](const SingleDerivedPath::Built & o) {
                    res += o.output;
                    res += '!';
                    toStringRest(*o.drvPath);
                },
            },
            p.raw());
    };

    std::visit(
        overloaded{
            [&](const HoffmanStringContextElem::Built & b) {
                res += '!';
                toStringRest(b);
            },
            [&](const HoffmanStringContextElem::Opaque & o) { toStringRest(o); },
            [&](const HoffmanStringContextElem::DrvDeep & d) {
                res += '=';
                res += d.drvPath.to_string();
            },
        },
        raw);

    return res;
}

std::string HoffmanStringContextElem::display(const StoreDirConfig & store) const
{
    return std::visit(
        overloaded{
            [&](const HoffmanStringContextElem::Opaque & o) -> std::string {
                return SingleDerivedPath{o}.to_string(store);
            },
            [&](const HoffmanStringContextElem::DrvDeep & d) -> std::string {
                return store.printStorePath(d.drvPath) + " (deep)";
            },
            [&](const HoffmanStringContextElem::Built & b) -> std::string { return SingleDerivedPath{b}.to_string(store); },
        },
        raw);
}

} // namespace hoffman
