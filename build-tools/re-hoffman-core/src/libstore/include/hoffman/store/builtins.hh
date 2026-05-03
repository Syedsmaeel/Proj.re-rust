#pragma once
///@file

#include "hoffman/store/derivations.hh"
#include "hoffman/store/config.hh"

#if HOFFMAN_WITH_AWS_AUTH
#  include "hoffman/store/aws-creds.hh"
#endif

namespace hoffman {

struct BuiltinBuilderContext
{
    const BasicDerivation & drv;
    std::map<std::string, std::string> outputs;
    std::string netrcData;
    std::string caFileData;
    Strings hashedMirrors;
    std::filesystem::path tmpDirInSandbox;

#if HOFFMAN_WITH_AWS_AUTH
    /**
     * Pre-resolved AWS credentials for S3 URLs in builtin:fetchurl.
     * When present, these should be used instead of creating new credential providers.
     */
    std::optional<AwsCredentials> awsCredentials;
#endif
};

using BuiltinBuilder = fun<void(const BuiltinBuilderContext &)>;

struct RegisterBuiltinBuilder
{
    typedef std::map<std::string, BuiltinBuilder> BuiltinBuilders;

    static BuiltinBuilders & builtinBuilders();

    RegisterBuiltinBuilder(const std::string & name, BuiltinBuilder && builder)
    {
        builtinBuilders().insert_or_assign(name, std::move(builder));
    }
};

} // namespace hoffman
