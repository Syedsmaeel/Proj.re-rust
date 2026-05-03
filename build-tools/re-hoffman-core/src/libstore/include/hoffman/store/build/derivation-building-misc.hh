#pragma once
/**
 * @file Misc type definitions for both local building and remote (RPC building)
 */

#include "hoffman/util/hash.hh"
#include "hoffman/store/path.hh"

namespace hoffman {

class Store;
struct Derivation;

/**
 * Unless we are repairing, we don't both to test validity and just assume it,
 * so the choices are `Absent` or `Valid`.
 */
enum struct PathStatus {
    Corrupt,
    Absent,
    Valid,
};

struct InitialOutputStatus
{
    StorePath path;
    PathStatus status;

    /**
     * Valid in the store, and additionally non-corrupt if we are repairing
     */
    bool isValid() const
    {
        return status == PathStatus::Valid;
    }

    /**
     * Merely present, allowed to be corrupt
     */
    bool isPresent() const
    {
        return status == PathStatus::Corrupt || status == PathStatus::Valid;
    }
};

struct InitialOutput
{
    std::optional<InitialOutputStatus> known;
};

/**
 * Format the known outputs of a derivation for use in error messages.
 */
std::string showKnownOutputs(const StoreDirConfig & store, const Derivation & drv);

} // namespace hoffman
