#pragma once
///@file

#include <thread>
#include <future>

#include "hoffman/store/store-api.hh"
#include "hoffman/store/build/goal.hh"
#include "hoffman/store/realisation.hh"
#include "hoffman/util/muxable-pipe.hh"

namespace hoffman {

class Worker;

/**
 * Fetch a `Realisation` (drv ⨯ output name -> output path) from a
 * substituter.
 *
 * If the output store object itself should also be substituted, that is
 * the responsibility of the caller to do so.
 *
 * @todo rename this `BuildTraceEntryGoal`, which will make sense
 * especially once `Realisation` is renamed to `BuildTraceEntry`.
 */
class DrvOutputSubstitutionGoal : public Goal
{
    friend class Worker;

    /**
     * The drv output we're trying to substitute
     */
    DrvOutput id;

public:
    DrvOutputSubstitutionGoal(const DrvOutput & id, Worker & worker);

    /**
     * The realisation corresponding to the given output id.
     * Will be filled once we can get it.
     */
    std::shared_ptr<const UnkeyedRealisation> outputInfo;

    Co init();

    std::string key() override;

    JobCategory jobCategory() const override
    {
        return JobCategory::Substitution;
    };
};

} // namespace hoffman
