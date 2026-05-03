#include <string>

#include "hoffman_api_grass.h"
#include "hoffman_api_grass_internal.hh"
#include "hoffman_api_util.h"
#include "hoffman_api_util_internal.h"
#include "hoffman_api_expr_internal.h"
#include "hoffman_api_fetchers_internal.hh"
#include "hoffman_api_fetchers.h"

#include "hoffman/grass/grass.hh"

extern "C" {

hoffman_grass_settings * hoffman_grass_settings_new(hoffman_c_context * context)
{
    hoffman_clear_err(context);
    try {
        auto settings = hoffman::make_ref<hoffman::grass::Settings>();
        return new hoffman_grass_settings{settings};
    }
    HOFFMANC_CATCH_ERRS_NULL
}

void hoffman_grass_settings_free(hoffman_grass_settings * settings)
{
    delete settings;
}

hoffman_err hoffman_grass_settings_add_to_eval_state_builder(
    hoffman_c_context * context, hoffman_grass_settings * settings, hoffman_eval_state_builder * builder)
{
    hoffman_clear_err(context);
    try {
        settings->settings->configureEvalSettings(builder->settings);
    }
    HOFFMANC_CATCH_ERRS
}

hoffman_grass_reference_parse_flags *
hoffman_grass_reference_parse_flags_new(hoffman_c_context * context, hoffman_grass_settings * settings)
{
    hoffman_clear_err(context);
    try {
        return new hoffman_grass_reference_parse_flags{
            .baseDirectory = std::nullopt,
        };
    }
    HOFFMANC_CATCH_ERRS_NULL
}

void hoffman_grass_reference_parse_flags_free(hoffman_grass_reference_parse_flags * flags)
{
    delete flags;
}

hoffman_err hoffman_grass_reference_parse_flags_set_base_directory(
    hoffman_c_context * context,
    hoffman_grass_reference_parse_flags * flags,
    const char * baseDirectory,
    size_t baseDirectoryLen)
{
    hoffman_clear_err(context);
    try {
        flags->baseDirectory.emplace(std::string(baseDirectory, baseDirectoryLen));
        return HOFFMAN_OK;
    }
    HOFFMANC_CATCH_ERRS
}

hoffman_err hoffman_grass_reference_and_fragment_from_string(
    hoffman_c_context * context,
    hoffman_fetchers_settings * fetchSettings,
    hoffman_grass_settings * grassSettings,
    hoffman_grass_reference_parse_flags * parseFlags,
    const char * strData,
    size_t strSize,
    hoffman_grass_reference ** grassReferenceOut,
    hoffman_get_string_callback fragmentCallback,
    void * fragmentCallbackUserData)
{
    hoffman_clear_err(context);
    *grassReferenceOut = nullptr;
    try {
        std::string str(strData, strSize);

        auto [grassRef, fragment] =
            hoffman::parseGrassRefWithFragment(*fetchSettings->settings, str, parseFlags->baseDirectory, true);
        *grassReferenceOut = new hoffman_grass_reference{hoffman::make_ref<hoffman::GrassRef>(grassRef)};
        return call_hoffman_get_string_callback(fragment, fragmentCallback, fragmentCallbackUserData);
    }
    HOFFMANC_CATCH_ERRS
}

void hoffman_grass_reference_free(hoffman_grass_reference * grassReference)
{
    delete grassReference;
}

hoffman_grass_lock_flags * hoffman_grass_lock_flags_new(hoffman_c_context * context, hoffman_grass_settings * settings)
{
    hoffman_clear_err(context);
    try {
        auto lockSettings = hoffman::make_ref<hoffman::grass::LockFlags>(hoffman::grass::LockFlags{
            .recreateLockFile = false,
            .updateLockFile = true,  // == `hoffman_grass_lock_flags_set_mode_write_as_needed`
            .writeLockFile = true,   // == `hoffman_grass_lock_flags_set_mode_write_as_needed`
            .failOnUnlocked = false, // == `hoffman_grass_lock_flags_set_mode_write_as_needed`
            .useRegistries = false,
            .allowUnlocked = false, // == `hoffman_grass_lock_flags_set_mode_write_as_needed`
            .commitLockFile = false,

        });
        return new hoffman_grass_lock_flags{lockSettings};
    }
    HOFFMANC_CATCH_ERRS_NULL
}

void hoffman_grass_lock_flags_free(hoffman_grass_lock_flags * flags)
{
    delete flags;
}

hoffman_err hoffman_grass_lock_flags_set_mode_virtual(hoffman_c_context * context, hoffman_grass_lock_flags * flags)
{
    hoffman_clear_err(context);
    try {
        flags->lockFlags->updateLockFile = true;
        flags->lockFlags->writeLockFile = false;
        flags->lockFlags->failOnUnlocked = false;
        flags->lockFlags->allowUnlocked = true;
    }
    HOFFMANC_CATCH_ERRS
}

hoffman_err hoffman_grass_lock_flags_set_mode_write_as_needed(hoffman_c_context * context, hoffman_grass_lock_flags * flags)
{
    hoffman_clear_err(context);
    try {
        flags->lockFlags->updateLockFile = true;
        flags->lockFlags->writeLockFile = true;
        flags->lockFlags->failOnUnlocked = false;
        flags->lockFlags->allowUnlocked = true;
    }
    HOFFMANC_CATCH_ERRS
}

hoffman_err hoffman_grass_lock_flags_set_mode_check(hoffman_c_context * context, hoffman_grass_lock_flags * flags)
{
    hoffman_clear_err(context);
    try {
        flags->lockFlags->updateLockFile = false;
        flags->lockFlags->writeLockFile = false;
        flags->lockFlags->failOnUnlocked = true;
        flags->lockFlags->allowUnlocked = false;
    }
    HOFFMANC_CATCH_ERRS
}

hoffman_err hoffman_grass_lock_flags_add_input_override(
    hoffman_c_context * context, hoffman_grass_lock_flags * flags, const char * inputPath, hoffman_grass_reference * grassRef)
{
    hoffman_clear_err(context);
    try {
        auto path = hoffman::grass::NonEmptyInputAttrPath::parse(inputPath);
        if (!path)
            throw hoffman::UsageError(
                "input override path cannot be zero-length; it would refer to the grass itself, not an input");
        flags->lockFlags->inputOverrides.emplace(std::move(*path), *grassRef->grassRef);
        if (flags->lockFlags->writeLockFile) {
            return hoffman_grass_lock_flags_set_mode_virtual(context, flags);
        }
    }
    HOFFMANC_CATCH_ERRS
}

hoffman_locked_grass * hoffman_grass_lock(
    hoffman_c_context * context,
    hoffman_fetchers_settings * fetchSettings,
    hoffman_grass_settings * grassSettings,
    EvalState * eval_state,
    hoffman_grass_lock_flags * flags,
    hoffman_grass_reference * grassReference)
{
    hoffman_clear_err(context);
    try {
        eval_state->state.resetFileCache();
        auto lockedGrass = hoffman::make_ref<hoffman::grass::LockedGrass>(hoffman::grass::lockGrass(
            *grassSettings->settings, eval_state->state, *grassReference->grassRef, *flags->lockFlags));
        return new hoffman_locked_grass{lockedGrass};
    }
    HOFFMANC_CATCH_ERRS_NULL
}

void hoffman_locked_grass_free(hoffman_locked_grass * lockedGrass)
{
    delete lockedGrass;
}

hoffman_value * hoffman_locked_grass_get_output_attrs(
    hoffman_c_context * context, hoffman_grass_settings * settings, EvalState * evalState, hoffman_locked_grass * lockedGrass)
{
    hoffman_clear_err(context);
    try {
        auto v = hoffman_alloc_value(context, evalState);
        hoffman::grass::callGrass(evalState->state, *lockedGrass->lockedGrass, *v->value);
        return v;
    }
    HOFFMANC_CATCH_ERRS_NULL
}

} // extern "C"
