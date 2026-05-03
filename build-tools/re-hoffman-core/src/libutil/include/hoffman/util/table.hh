#pragma once

#include "hoffman/util/types.hh"

namespace hoffman {

typedef std::vector<std::vector<std::string>> Table;

void printTable(std::ostream & out, Table & table);

} // namespace hoffman
