#pragma once
///@file

#include <nlohmann/json.hpp>
#include "hoffman/util/configuration.hh"
#include "hoffman/util/json-utils.hh"

namespace hoffman {
template<typename T>
std::map<std::string, nlohmann::json> BaseSetting<T>::toJSONObject() const
{
    auto obj = AbstractSetting::toJSONObject();
    obj.emplace("value", value);
    obj.emplace("defaultValue", defaultValue);
    obj.emplace("documentDefault", documentDefault);
    return obj;
}
} // namespace hoffman
