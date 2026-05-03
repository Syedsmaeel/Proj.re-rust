#include "hoffman/util/json-utils.hh"
#include "hoffman/store/build/derivation-builder.hh"

namespace nlohmann {

using namespace hoffman;

ExternalBuilder adl_serializer<ExternalBuilder>::from_json(const json & json)
{
    auto obj = getObject(json);
    return {
        .systems = valueAt(obj, "systems"),
        .program = valueAt(obj, "program"),
        .args = valueAt(obj, "args"),
    };
}

void adl_serializer<ExternalBuilder>::to_json(json & json, const ExternalBuilder & eb)
{
    json = {
        {"systems", eb.systems},
        {"program", eb.program},
        {"args", eb.args},
    };
}

} // namespace nlohmann
