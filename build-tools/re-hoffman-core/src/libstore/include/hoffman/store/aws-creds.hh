#pragma once
///@file
#include "hoffman/store/config.hh"

#if HOFFMAN_WITH_AWS_AUTH

#  include "hoffman/store/s3-url.hh"
#  include "hoffman/util/ref.hh"
#  include "hoffman/util/error.hh"

#  include <memory>
#  include <optional>
#  include <string>

namespace hoffman {

/**
 * AWS credentials obtained from credential providers
 */
struct AwsCredentials
{
    std::string accessKeyId;
    std::string secretAccessKey;
    std::optional<std::string> sessionToken;

    AwsCredentials(
        const std::string & accessKeyId,
        const std::string & secretAccessKey,
        const std::optional<std::string> & sessionToken = std::nullopt)
        : accessKeyId(accessKeyId)
        , secretAccessKey(secretAccessKey)
        , sessionToken(sessionToken)
    {
    }
};

class AwsAuthError final : public CloneableError<AwsAuthError, Error>
{
    std::optional<int> errorCode;

public:
    using CloneableError::CloneableError;

    AwsAuthError(int errorCode);

    std::optional<int> getErrorCode() const
    {
        return errorCode;
    }
};

class AwsCredentialProvider
{
public:
    /**
     * Get AWS credentials for the given URL.
     *
     * @param url The S3 url to get the credentials for
     * @return AWS credentials
     * @throws AwsAuthError if credentials cannot be resolved
     */
    virtual AwsCredentials getCredentials(const ParsedS3URL & url) = 0;

    std::optional<AwsCredentials> maybeGetCredentials(const ParsedS3URL & url)
    {
        try {
            return getCredentials(url);
        } catch (AwsAuthError & e) {
            return std::nullopt;
        }
    }

    virtual ~AwsCredentialProvider() {}
};

/**
 * Create a new instancee of AwsCredentialProvider.
 */
ref<AwsCredentialProvider> makeAwsCredentialsProvider();

/**
 * Get a reference to the global AwsCredentialProvider.
 */
ref<AwsCredentialProvider> getAwsCredentialsProvider();

} // namespace hoffman
#endif
