#pragma once

#ifndef BISON_HEADER
#  include "parser-tab.hh"
using YYSTYPE = hoffman::parser::BisonParser::value_type;
using YYLTYPE = hoffman::parser::BisonParser::location_type;
#  include "lexer-tab.hh" // IWYU pragma: export
#endif

namespace hoffman {

class Parser : public parser::BisonParser
{
    using BisonParser::BisonParser;
};

} // namespace hoffman
