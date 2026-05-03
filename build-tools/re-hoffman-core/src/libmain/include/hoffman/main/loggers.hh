#pragma once
///@file

#include "hoffman/util/types.hh"

namespace hoffman {

enum class LogFormat {
    raw,
    rawWithLogs,
    internalJSON,
    bar,
    barWithLogs,
};

void setLogFormat(const std::string & logFormatStr);
void setLogFormat(const LogFormat & logFormat);

} // namespace hoffman
