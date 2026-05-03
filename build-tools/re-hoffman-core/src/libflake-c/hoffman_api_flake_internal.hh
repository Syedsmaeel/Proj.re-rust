#pragma once
#include <optional>

#include "hoffman/util/ref.hh"
#include "hoffman/flake/flake.hh"
#include "hoffman/flake/flakeref.hh"
#include "hoffman/flake/settings.hh"

struct hoffman_flake_settings
{
    hoffman::ref<hoffman::flake::Settings> settings;
};

struct hoffman_flake_reference_parse_flags
{
    std::optional<std::filesystem::path> baseDirectory;
};

struct hoffman_flake_reference
{
    hoffman::ref<hoffman::FlakeRef> flakeRef;
};

struct hoffman_flake_lock_flags
{
    hoffman::ref<hoffman::flake::LockFlags> lockFlags;
};

struct hoffman_locked_flake
{
    hoffman::ref<hoffman::flake::LockedFlake> lockedFlake;
};
