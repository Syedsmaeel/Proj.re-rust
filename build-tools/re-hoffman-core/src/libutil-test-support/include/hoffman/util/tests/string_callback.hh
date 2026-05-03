#pragma once
#include <string>

namespace hoffman::testing {

void observe_string_cb(const char * start, unsigned int n, void * user_data);

inline void * observe_string_cb_data(std::string & out)
{
    return (void *) &out;
};

#define OBSERVE_STRING(str) hoffman::testing::observe_string_cb, hoffman::testing::observe_string_cb_data(str)

} // namespace hoffman::testing
