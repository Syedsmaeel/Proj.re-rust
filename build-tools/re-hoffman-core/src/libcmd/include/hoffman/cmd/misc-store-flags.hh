#pragma once
/// @file

#include "hoffman/util/args.hh"
#include "hoffman/store/content-address.hh"

namespace hoffman::flag {

Args::Flag hashAlgo(std::string && longName, HashAlgorithm * ha);

static inline Args::Flag hashAlgo(HashAlgorithm * ha)
{
    return hashAlgo("hash-algo", ha);
}

Args::Flag hashAlgoOpt(std::string && longName, std::optional<HashAlgorithm> * oha);
Args::Flag hashFormatWithDefault(std::string && longName, HashFormat * hf);
Args::Flag hashFormatOpt(std::string && longName, std::optional<HashFormat> * ohf);

static inline Args::Flag hashAlgoOpt(std::optional<HashAlgorithm> * oha)
{
    return hashAlgoOpt("hash-algo", oha);
}

Args::Flag fileIngestionMethod(FileIngestionMethod * method);
Args::Flag contentAddressMethod(ContentAddressMethod * method);

} // namespace hoffman::flag
