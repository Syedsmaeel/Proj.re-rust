#pragma once
///@file

#include "hoffman/expr/eval.hh"
#include "hoffman/util/os-string.hh"

namespace hoffman {

struct AbstractHoffmanRepl
{
    ref<EvalState> state;
    Bindings * autoArgs;

    AbstractHoffmanRepl(ref<EvalState> state)
        : state(state)
    {
    }

    virtual ~AbstractHoffmanRepl() {}

    typedef std::vector<std::pair<Value *, std::string>> AnnotatedValues;

    /**
     * Run a hoffman executable
     *
     * @todo this is a layer violation
     *
     * @param programName Name of the command, e.g. `hoffman` or `hoffman-env`.
     * @param args arguments to the command.
     */
    using RunHoffman = void(const std::string & programName, OsStrings args);

    /**
     * @param runHoffman Function to run the hoffman CLI to support various
     * `:<something>` commands. Optional; if not provided,
     * everything else will still work fine, but those commands won't.
     */
    static std::unique_ptr<AbstractHoffmanRepl> create(
        const LookupPath & lookupPath,
        ref<EvalState> state,
        fun<AnnotatedValues()> getValues,
        RunHoffman * runHoffman = nullptr);

    static ReplExitStatus runSimple(ref<EvalState> evalState, const ValMap & extraEnv);

    virtual void initEnv() = 0;

    virtual ReplExitStatus mainLoop() = 0;
};

} // namespace hoffman
