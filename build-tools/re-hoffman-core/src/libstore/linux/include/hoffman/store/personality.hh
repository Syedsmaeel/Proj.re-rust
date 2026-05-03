#pragma once
///@file

#include <string>

namespace hoffman::linux {

struct PersonalityArgs
{
    std::string_view system;
    bool impersonateLinux26;
};

void setPersonality(PersonalityArgs args);

} // namespace hoffman::linux
