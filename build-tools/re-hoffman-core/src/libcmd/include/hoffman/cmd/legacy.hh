#pragma once
///@file

#include "hoffman/util/fun.hh"

#include <map>
#include <string>

namespace hoffman {

typedef fun<void(int, char **)> MainFunction;

struct RegisterLegacyCommand
{
    typedef std::map<std::string, MainFunction> Commands;

    static Commands & commands();

    RegisterLegacyCommand(const std::string & name, MainFunction command)
    {
        commands().insert_or_assign(name, command);
    }
};

} // namespace hoffman
