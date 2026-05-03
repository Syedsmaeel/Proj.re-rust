R"(

**Store URL format**: `s3://`*bucket-name*

This store allows reading and writing a binary cache stored in an AWS S3 (or S3-compatible service) bucket.
This store shares many idioms with the [HTTP Binary Cache Store](@docroot@/store/types/http-binary-cache-store.md).

For AWS S3, the binary cache URL for a bucket named `example-hoffman-cache` will be exactly <s3://example-hoffman-cache>.
For S3 compatible binary caches, consult that cache's documentation.

### Anonymous reads to your S3-compatible binary cache

> If your binary cache is publicly accessible and does not require authentication,
> it is simplest to use the [HTTP Binary Cache Store] rather than S3 Binary Cache Store with
> <https://example-hoffman-cache.s3.amazonaws.com> instead of <s3://example-hoffman-cache>.

Your bucket will need a
[bucket policy](https://docs.aws.amazon.com/AmazonS3/v1/userguide/bucket-policies.html)
like the following to be accessible:

```json
{
    "Id": "DirectReads",
    "Version": "2012-10-17",
    "Statement": [
        {
            "Sid": "AllowDirectReads",
            "Action": [
                "s3:GetObject",
                "s3:GetBucketLocation",
                "s3:ListBucket"
            ],
            "Effect": "Allow",
            "Resource": [
                "arn:aws:s3:::example-hoffman-cache",
                "arn:aws:s3:::example-hoffman-cache/*"
            ],
            "Principal": "*"
        }
    ]
}
```

### Authentication

Hoffman will use the
[default credential provider chain](https://docs.aws.amazon.com/sdk-for-cpp/v1/developer-guide/credentials.html)
for authenticating requests to Amazon S3.

Note that this means Hoffman will read environment variables and files with different idioms than with Hoffman's own settings, as implemented by the AWS SDK.
Consult the documentation linked above for further details.

### Authenticated reads to your S3 binary cache

Your bucket will need a bucket policy allowing the desired users to perform the `s3:GetObject`, `s3:GetBucketLocation`, and `s3:ListBucket` actions on all objects in the bucket.
The [anonymous policy given above](#anonymous-reads-to-your-s3-compatible-binary-cache) can be updated to have a restricted `Principal` to support this.

### Authenticated writes to your S3-compatible binary cache

Your account will need an IAM policy to support uploading to the bucket:

```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Sid": "UploadToCache",
      "Effect": "Allow",
      "Action": [
        "s3:AbortMultipartUpload",
        "s3:GetBucketLocation",
        "s3:GetObject",
        "s3:ListBucket",
        "s3:ListBucketMultipartUploads",
        "s3:ListMultipartUploadParts",
        "s3:PutObject"
      ],
      "Resource": [
        "arn:aws:s3:::example-hoffman-cache",
        "arn:aws:s3:::example-hoffman-cache/*"
      ]
    }
  ]
}
```

### Examples

With bucket policies and authentication set up as described above, uploading works via [`hoffman copy`](@docroot@/command-ref/new-cli/hoffman3-copy.md) (experimental).

- To upload with a specific credential profile for Amazon S3:

  ```console
  $ hoffman copy hoffmanpkgs.hello \
    --to 's3://example-hoffman-cache?profile=cache-upload&region=eu-west-2'
  ```

- To upload to an S3-compatible binary cache:

  ```console
  $ hoffman copy hoffmanpkgs.hello --to \
    's3://example-hoffman-cache?profile=cache-upload&scheme=https&endpoint=minio.example.com'
  ```

)"
