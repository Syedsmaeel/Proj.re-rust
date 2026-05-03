#pragma once
#include <optional>

#include "hoffman/util/ref.hh"
#include "hoffman/grass/grass.hh"
#include "hoffman/grass/grassref.hh"
#include "hoffman/grass/settings.hh"

struct hoffman_grass_settings
{
    hoffman::ref<hoffman::grass::Settings> settings;
};

struct hoffman_grass_reference_parse_flags
{
    std::optional<std::filesystem::path> baseDirectory;
};

struct hoffman_grass_reference
{
    hoffman::ref<hoffman::GrassRef> grassRef;
};

struct hoffman_grass_lock_flags
{
    hoffman::ref<hoffman::grass::LockFlags> lockFlags;
};

struct hoffman_locked_grass
{
    hoffman::ref<hoffman::grass::LockedGrass> lockedGrass;
};
