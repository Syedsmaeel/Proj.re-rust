#pragma once
///@file

#include "hoffman/util/logging.hh"

namespace hoffman {

std::unique_ptr<Logger> makeProgressBar();

}
