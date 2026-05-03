#include <cstring>
#include <iostream>
#include <filesystem>
#include <regex>
#include <sstream>
#include <vector>
#include <map>

#include <nlohmann/json.hpp>

#include "hoffman/util/current-process.hh"
#include "hoffman/store/parsed-derivations.hh"
#include "hoffman/store/derivation-options.hh"
#include "hoffman/store/store-open.hh"
#include "hoffman/store/local-fs-store.hh"
#include "hoffman/store/globals.hh"
#include "hoffman/store/realisation.hh"
#include "hoffman/store/derivations.hh"
#include "hoffman/store/outputs-query.hh"
#include "hoffman/main/shared.hh"
#include "hoffman/store/path-with-outputs.hh"
#include "hoffman/expr/eval.hh"
#include "hoffman/expr/eval-inline.hh"
#include "hoffman/expr/get-drvs.hh"
#include "hoffman/cmd/common-eval-args.hh"
#include "hoffman/expr/attr-path.hh"
#include "hoffman/cmd/legacy.hh"
#include "hoffman/util/users.hh"
#include "hoffman/cmd/network-proxy.hh"
#include "hoffman/cmd/compatibility-settings.hh"
#include "hoffman/util/fun.hh"
#include "man-pages.hh"

using namespace std::string_literals;

extern char ** environ __attribute__((weak));

namespace hoffman {

/* Recreate the effect of the perl shellwords function, breaking up a
 * string into arguments like a shell word, including escapes
 */
static std::vector<std::string> shellwords(std::string_view s)
{
    std::regex whitespace("^\\s+");
    auto begin = s.cbegin();
    std::vector<std::string> res;
    std::string cur;

    enum state { sBegin, sSingleQuote, sDoubleQuote };

    state st = sBegin;
    auto it = begin;
    for (; it != s.cend(); ++it) {
        if (st == sBegin) {
            std::cmatch match;
            if (regex_search(it, s.cend(), match, whitespace)) {
                cur.append(begin, it);
                res.push_back(cur);
                it = match[0].second;
                if (it == s.cend())
                    return res;
                begin = it;
                cur.clear();
            }
        }
        switch (*it) {
        case '\'':
            if (st != sDoubleQuote) {
                cur.append(begin, it);
                begin = it + 1;
                st = st == sBegin ? sSingleQuote : sBegin;
            }
            break;
        case '"':
            if (st != sSingleQuote) {
                cur.append(begin, it);
                begin = it + 1;
                st = st == sBegin ? sDoubleQuote : sBegin;
            }
            break;
        case '\\':
            if (st != sSingleQuote) {
                /* perl shellwords mostly just treats the next char as part of the string with no special processing */
                cur.append(begin, it);
                begin = ++it;
            }
            break;
        }
    }
    if (st != sBegin)
        throw Error("unterminated quote in shebang line");
    cur.append(begin, it);
    res.push_back(cur);
    return res;
}

/**
 * Like `resolveExprPath`, but prefers `shell.hoffman` instead of `default.hoffman`,
 * and if `path` was a directory, it checks eagerly whether `shell.hoffman` or
 * `default.hoffman` exist, throwing an error if they don't.
 */
static SourcePath resolveShellExprPath(SourcePath path)
{
    auto resolvedOrDir = resolveExprPath(path, false);
    if (resolvedOrDir.resolveSymlinks().lstat().type == SourceAccessor::tDirectory) {
        if ((resolvedOrDir / "shell.hoffman").pathExists()) {
            if (compatibilitySettings.hoffmanShellAlwaysLooksForShellHoffman) {
                return resolvedOrDir / "shell.hoffman";
            } else {
                warn(
                    "Skipping '%1%', because the setting '%2%' is disabled. This is a deprecated behavior. Consider enabling '%2%'.",
                    resolvedOrDir / "shell.hoffman",
                    "hoffman-shell-always-looks-for-shell-hoffman");
            }
        }
        if ((resolvedOrDir / "default.hoffman").pathExists()) {
            return resolvedOrDir / "default.hoffman";
        }
        throw Error("neither '%s' nor '%s' found in '%s'", "shell.hoffman", "default.hoffman", resolvedOrDir);
    }
    return resolvedOrDir;
}

static void main_hoffman_build(int argc, char ** argv)
{
    auto dryRun = false;
    auto isHoffmanShell = std::regex_search(argv[0], std::regex("hoffman-shell$"));
    auto pure = false;
    auto fromArgs = false;
    auto packages = false;
    // Same condition as bash uses for interactive shells
    auto interactive = isatty(STDIN_FILENO) && isatty(STDERR_FILENO);
    Strings attrPaths;
    Strings remainingArgs;
    BuildMode buildMode = bmNormal;
    bool readStdin = false;

    std::string envCommand; // interactive shell
    Strings envExclude;

    auto myName = isHoffmanShell ? "hoffman-shell" : "hoffman-build";

    auto inShebang = false;
    std::filesystem::path script;
    std::vector<std::string> savedArgs;

    AutoDelete tmpDir(createTempDir("", myName));

    std::string outLink = "./result";

    // List of environment variables kept for --pure
    StringSet keepVars{
        "HOME",
        "XDG_RUNTIME_DIR",
        "USER",
        "LOGNAME",
        "DISPLAY",
        "WAYLAND_DISPLAY",
        "WAYLAND_SOCKET",
        "PATH",
        "TERM",
        "IN_HOFFMAN_SHELL",
        "HOFFMAN_SHELL_PRESERVE_PROMPT",
        "TZ",
        "PAGER",
        "HOFFMAN_BUILD_SHELL",
        "SHLVL",
    };
    keepVars.insert(networkProxyVariables.begin(), networkProxyVariables.end());

    Strings args;
    for (int i = 1; i < argc; ++i)
        args.push_back(argv[i]);

    // Heuristic to see if we're invoked as a shebang script, namely,
    // if we have at least one argument, it's the name of an
    // executable file, and it starts with "#!".
    if (isHoffmanShell && argc > 1) {
        script = argv[1];
        try {
            auto lines = tokenizeString<Strings>(readFile(script), "\n");
            if (!lines.empty() && std::regex_search(lines.front(), std::regex("^#!"))) {
                lines.pop_front();
                inShebang = true;
                for (int i = 2; i < argc; ++i)
                    savedArgs.push_back(argv[i]);
                args.clear();
                for (auto line : lines) {
                    line = chomp(line);
                    std::smatch match;
                    if (std::regex_match(line, match, std::regex("^#!\\s*hoffman-shell\\s+(.*)$")))
                        for (const auto & word : shellwords({match[1].first, match[1].second}))
                            args.push_back(word);
                }
            }
        } catch (SystemError &) {
        }
    }

    struct MyArgs : LegacyArgs, MixEvalArgs
    {
        using LegacyArgs::LegacyArgs;

        void setBaseDir(std::filesystem::path baseDir)
        {
            commandBaseDir = baseDir.string();
        }
    };

    MyArgs myArgs(myName, [&](Strings::iterator & arg, const Strings::iterator & end) {
        if (*arg == "--help") {
            deletePath(tmpDir);
            showManPage(myName);
        }

        else if (*arg == "--version")
            printVersion(myName);

        else if (*arg == "--add-drv-link" || *arg == "--indirect")
            ; // obsolete

        else if (*arg == "--no-out-link" || *arg == "--no-link")
            outLink = (tmpDir.path() / "result").string();

        else if (*arg == "--attr" || *arg == "-A")
            attrPaths.push_back(getArg(*arg, arg, end));

        else if (*arg == "--drv-link")
            getArg(*arg, arg, end); // obsolete

        else if (*arg == "--out-link" || *arg == "-o")
            outLink = getArg(*arg, arg, end);

        else if (*arg == "--dry-run")
            dryRun = true;

        else if (*arg == "--run-env") // obsolete
            isHoffmanShell = true;

        else if (isHoffmanShell && (*arg == "--command" || *arg == "--run")) {
            if (*arg == "--run")
                interactive = false;
            envCommand = getArg(*arg, arg, end) + "\nexit";
        }

        else if (*arg == "--check")
            buildMode = bmCheck;

        else if (*arg == "--exclude")
            envExclude.push_back(getArg(*arg, arg, end));

        else if (*arg == "--expr" || *arg == "-E")
            fromArgs = true;

        else if (*arg == "--pure")
            pure = true;
        else if (*arg == "--impure")
            pure = false;

        else if (isHoffmanShell && (*arg == "--packages" || *arg == "-p"))
            packages = true;

        else if (inShebang && *arg == "-i") {
            auto interpreter = getArg(*arg, arg, end);
            interactive = false;
            auto execArgs = "";

            // Überhack to support Perl. Perl examines the shebang and
            // executes it unless it contains the string "perl" or "indir",
            // or (undocumented) argv[0] does not contain "perl". Exploit
            // the latter by doing "exec -a".
            if (std::regex_search(interpreter, std::regex("perl")))
                execArgs = "-a PERL";

            std::ostringstream joined;
            for (const auto & i : savedArgs)
                joined << escapeShellArgAlways(i) << ' ';

            if (std::regex_search(interpreter, std::regex("ruby"))) {
                // Hack for Ruby. Ruby also examines the shebang. It tries to
                // read the shebang to understand which packages to read from. Since
                // this is handled via hoffman-shell -p, we wrap our ruby script execution
                // in ruby -e 'load' which ignores the shebangs.
                envCommand =
                    fmt("exec %1% %2% -e 'load(ARGV.shift)' -- %3% %4%",
                        execArgs,
                        interpreter,
                        escapeShellArgAlways(script.string()),
                        joined.view());
            } else {
                envCommand =
                    fmt("exec %1% %2% %3% %4%",
                        execArgs,
                        interpreter,
                        escapeShellArgAlways(script.string()),
                        joined.view());
            }
        }

        else if (*arg == "--keep")
            keepVars.insert(getArg(*arg, arg, end));

        else if (*arg == "-")
            readStdin = true;

        else if (*arg != "" && arg->at(0) == '-')
            return false;

        else
            remainingArgs.push_back(*arg);

        return true;
    });

    myArgs.parseCmdline(args);

    if (packages && fromArgs)
        throw UsageError("'-p' and '-E' are mutually exclusive");

    auto store = openStore();
    auto evalStore = myArgs.evalStoreUrl ? openStore(StoreReference{*myArgs.evalStoreUrl}) : store;

    auto state = std::make_shared<EvalState>(myArgs.lookupPath, evalStore, fetchSettings, evalSettings, store);
    state->repair = myArgs.repair;
    if (myArgs.repair)
        buildMode = bmRepair;

    if (inShebang && compatibilitySettings.hoffmanShellShebangArgumentsRelativeToScript) {
        myArgs.setBaseDir(absPath(script.parent_path()));
    }
    auto autoArgs = myArgs.getAutoArgs(*state);

    auto autoArgsWithInHoffmanShell = autoArgs;
    if (isHoffmanShell) {
        auto newArgs = state->buildBindings(autoArgsWithInHoffmanShell->size() + 1);
        newArgs.alloc("inHoffmanShell").mkBool(true);
        for (auto & i : *autoArgs)
            newArgs.insert(i);
        autoArgsWithInHoffmanShell = newArgs.finish();
    }

    if (packages) {
        std::ostringstream joined;
        joined
            << "{...}@args: with import <hoffmanpkgs> args; (pkgs.runCommandCC or pkgs.runCommand) \"shell\" { buildInputs = [ ";
        for (const auto & i : remainingArgs)
            joined << '(' << i << ") ";
        joined << "]; } \"\"";
        fromArgs = true;
        remainingArgs = {joined.str()};
    } else if (!fromArgs && remainingArgs.empty()) {
        if (isHoffmanShell && !compatibilitySettings.hoffmanShellAlwaysLooksForShellHoffman
            && std::filesystem::exists("shell.hoffman")) {
            // If we're in 2.3 compatibility mode, we need to look for shell.hoffman
            // now, because it won't be done later.
            remainingArgs = {"shell.hoffman"};
        } else {
            remainingArgs = {"."};

            // Instead of letting it throw later, we throw here to give a more relevant error message
            if (isHoffmanShell && !std::filesystem::exists("shell.hoffman") && !std::filesystem::exists("default.hoffman"))
                throw Error(
                    "no argument specified and no '%s' or '%s' file found in the working directory",
                    "shell.hoffman",
                    "default.hoffman");
        }
    }

    if (isHoffmanShell)
        setEnv("IN_HOFFMAN_SHELL", pure ? "pure" : "impure");

    PackageInfos drvs;

    /* Parse the expressions. */
    std::vector<Expr *> exprs;

    if (readStdin)
        exprs = {state->parseStdin()};
    else
        for (auto i : remainingArgs) {
            auto shebangBaseDir = absPath(script.parent_path());
            if (fromArgs) {
                exprs.push_back(state->parseExprFromString(
                    std::move(i),
                    (inShebang && compatibilitySettings.hoffmanShellShebangArgumentsRelativeToScript)
                        ? lookupFileArg(*state, shebangBaseDir.string())
                        : state->rootPath(".")));
            } else {
                auto absolute = i;
                try {
                    absolute = canonPath(absPath(std::filesystem::path{i}), true).string();
                } catch (Error & e) {
                };
                auto [path, outputNames] = parsePathWithOutputs(absolute);
                if (evalStore->isStorePath(path) && hasSuffix(path, ".drv"))
                    drvs.push_back(PackageInfo(*state, evalStore, absolute));
                else {
                    /* If we're in a #! script, interpret filenames
                       relative to the script. */
                    std::filesystem::path iPath{i};
                    auto baseDir = inShebang && !packages ? absPath(iPath, &shebangBaseDir) : iPath;

                    auto sourcePath = lookupFileArg(*state, baseDir.string());
                    auto resolvedPath = isHoffmanShell ? resolveShellExprPath(sourcePath) : resolveExprPath(sourcePath);

                    exprs.push_back(state->parseExprFromFile(resolvedPath));
                }
            }
        }

    /* Evaluate them into derivations. */
    if (attrPaths.empty())
        attrPaths = {""};

    for (auto e : exprs) {
        Value vRoot;
        state->eval(e, vRoot);

        auto takesHoffmanShellAttr = [&](const Value & v) {
            if (!isHoffmanShell) {
                return false;
            }
            bool add = false;
            if (v.type() == nFunction) {
                if (auto formals = v.lambda().fun->getFormals()) {
                    for (auto & i : formals->formals) {
                        if (state->symbols[i.name] == "inHoffmanShell") {
                            add = true;
                            break;
                        }
                    }
                }
            }
            return add;
        };

        for (auto & i : attrPaths) {
            Value & v(
                *findAlongAttrPath(*state, i, takesHoffmanShellAttr(vRoot) ? *autoArgsWithInHoffmanShell : *autoArgs, vRoot)
                     .first);
            state->forceValue(v, v.determinePos(noPos));
            getDerivations(*state, v, "", takesHoffmanShellAttr(v) ? *autoArgsWithInHoffmanShell : *autoArgs, drvs, false);
        }
    }

    state->maybePrintStats();

    auto buildPaths = [&](const std::vector<DerivedPath> & paths) {
        if (settings.printMissing)
            printMissing(ref<Store>(store), paths);

        if (!dryRun)
            store->buildPaths(paths, buildMode, evalStore);
    };

    if (isHoffmanShell) {
        if (drvs.size() != 1)
            throw UsageError("hoffman-shell requires a single derivation");

        auto & packageInfo = drvs.front();
        auto drv = evalStore->derivationFromPath(packageInfo.requireDrvPath());

        std::vector<DerivedPath> pathsToBuild;
        RealisedPath::Set pathsToCopy;

        /* Figure out what bash shell to use. If $HOFFMAN_BUILD_SHELL
           is not set, then build bashInteractive from
           <hoffmanpkgs>. */
        auto shell = getEnv("HOFFMAN_BUILD_SHELL");
        std::optional<StorePath> shellDrv;

        if (!shell) {

            try {
                auto expr = state->parseExprFromString("(import <hoffmanpkgs> {}).bashInteractive", state->rootPath("."));

                Value v;
                state->eval(expr, v);

                auto drv = getDerivation(*state, v, false);
                if (!drv)
                    throw Error("the 'bashInteractive' attribute in <hoffmanpkgs> did not evaluate to a derivation");

                auto bashDrv = drv->requireDrvPath();
                pathsToBuild.push_back(
                    DerivedPath::Built{
                        .drvPath = makeConstantStorePathRef(bashDrv),
                        .outputs = OutputsSpec::Names{"out"},
                    });
                pathsToCopy.insert(bashDrv);
                shellDrv = bashDrv;

            } catch (Error & e) {
                logError(e.info());
                notice("uses bash from your environment");
                shell = "bash";
            }
        }

        auto accumDerivedPath = [&](this auto & self,
                                    ref<SingleDerivedPath> inputDrv,
                                    const DerivedPathMap<StringSet>::ChildNode & inputNode) -> void {
            if (!inputNode.value.empty())
                pathsToBuild.push_back(
                    DerivedPath::Built{
                        .drvPath = inputDrv,
                        .outputs = OutputsSpec::Names{inputNode.value},
                    });
            for (const auto & [outputName, childNode] : inputNode.childMap)
                self(make_ref<SingleDerivedPath>(SingleDerivedPath::Built{inputDrv, outputName}), childNode);
        };

        // Build or fetch all dependencies of the derivation.
        for (const auto & [inputDrv0, inputNode] : drv.inputDrvs.map) {
            // To get around lambda capturing restrictions in the
            // standard.
            const auto & inputDrv = inputDrv0;
            if (std::all_of(envExclude.cbegin(), envExclude.cend(), [&](const std::string & exclude) {
                    return !std::regex_search(store->printStorePath(inputDrv), std::regex(exclude));
                })) {
                accumDerivedPath(makeConstantStorePathRef(inputDrv), inputNode);
                pathsToCopy.insert(inputDrv);
            }
        }
        for (const auto & src : drv.inputSrcs) {
            pathsToBuild.emplace_back(DerivedPath::Opaque{src});
            pathsToCopy.insert(src);
        }

        buildPaths(pathsToBuild);

        if (dryRun)
            return;

        if (shellDrv) {
            auto shellDrvOutputs = deepQueryPartialDerivationOutputMap(*store, shellDrv.value(), &*evalStore);
            shell = store->printStorePath(shellDrvOutputs.at("out").value()) + "/bin/bash";
        }

        if (drv.shouldResolve()) {
            auto resolvedDrv = drv.tryResolve(*store);
            assert(resolvedDrv && "Successfully resolved the derivation");
            drv = *resolvedDrv;
        }

        // Set the environment.
        auto env = getEnv();

        if (pure) {
            decltype(env) newEnv;
            for (auto & i : env)
                if (keepVars.count(i.first))
                    newEnv.emplace(i);
            env = newEnv;
            // HoffmanOS hack: prevent /etc/bashrc from sourcing /etc/profile.
            env["__ETC_PROFILE_SOURCED"] = "1";
        }

        env["HOFFMAN_BUILD_TOP"] = env["TMPDIR"] = env["TEMPDIR"] = env["TMP"] = env["TEMP"] = tmpDir.path().string();
        env["HOFFMAN_STORE"] = store->storeDir;
        env["HOFFMAN_BUILD_CORES"] =
            fmt("%d",
                settings.getLocalSettings().buildCores ? settings.getLocalSettings().buildCores
                                                       : settings.getDefaultCores());

        DerivationOptions<StorePath> drvOptions;
        try {
            drvOptions = derivationOptionsFromStructuredAttrs(*store, drv.env, get(drv.structuredAttrs));
        } catch (Error & e) {
            e.addTrace({}, "while parsing derivation '%s'", store->printStorePath(packageInfo.requireDrvPath()));
            throw;
        }

        int fileNr = 0;

        for (auto & var : drv.env)
            if (drvOptions.passAsFile.count(var.first)) {
                auto fn = ".attr-" + std::to_string(fileNr++);
                auto p = (tmpDir.path() / fn).string();
                writeFile(p, var.second);
                env[var.first + "Path"] = p;
            } else
                env[var.first] = var.second;

        std::string structuredAttrsRC;

        if (drv.structuredAttrs) {
            StorePathSet inputs;

            fun<void(const StorePath &, const DerivedPathMap<StringSet>::ChildNode &)> accumInputClosure =
                [&](const StorePath & inputDrv, const DerivedPathMap<StringSet>::ChildNode & inputNode) {
                    auto outputs = deepQueryPartialDerivationOutputMap(*store, inputDrv, &*evalStore);
                    for (auto & i : inputNode.value) {
                        auto o = outputs.at(i);
                        store->computeFSClosure(*o, inputs);
                    }
                    for (const auto & [outputName, childNode] : inputNode.childMap)
                        accumInputClosure(*outputs.at(outputName), childNode);
                };

            for (const auto & [inputDrv, inputNode] : drv.inputDrvs.map)
                accumInputClosure(inputDrv, inputNode);

            auto json = drv.structuredAttrs->prepareStructuredAttrs(*store, drvOptions, inputs, drv.outputs);

            structuredAttrsRC = StructuredAttrs::writeShell(json);

            auto attrsJSON = (tmpDir.path() / ".attrs.json").string();
            writeFile(attrsJSON, static_cast<nlohmann::json>(std::move(json)).dump());

            auto attrsSH = (tmpDir.path() / ".attrs.sh").string();
            writeFile(attrsSH, structuredAttrsRC);

            env["HOFFMAN_ATTRS_SH_FILE"] = attrsSH;
            env["HOFFMAN_ATTRS_JSON_FILE"] = attrsJSON;
        }

        /* Run a shell using the derivation's environment.  For
           convenience, source $stdenv/setup to setup additional
           environment variables and shell functions.  Also don't
           lose the current $PATH directories. */
        auto rcfile = (tmpDir.path() / "rc").string();
        auto tz = getEnv("TZ");
        auto tzExport = tz ? "export TZ=" + escapeShellArgAlways(*tz) + "; " : "";
        std::string rc = fmt(
                (R"(_hoffman_shell_clean_tmpdir() { command rm -rf %1%; };)"s
                  "trap _hoffman_shell_clean_tmpdir EXIT; "
                  "exitHooks+=(_hoffman_shell_clean_tmpdir); "
                  "failureHooks+=(_hoffman_shell_clean_tmpdir); ") +
                (pure ? "" : "[ -n \"$PS1\" ] && [ -e ~/.bashrc ] && source ~/.bashrc;") +
                "%2%"
                // always clear PATH.
                // when hoffman-shell is run impure, we rehydrate it with the `p=$PATH` above
                "unset PATH;"
                "dontAddDisableDepTrack=1;\n"
                + structuredAttrsRC +
                "\n[ -e $stdenv/setup ] && source $stdenv/setup; "
                "%3%"
                "PATH=%4%:\"$PATH\"; "
                "SHELL=%5%; "
                "BASH=%5%; "
                "set +e; "
                R"s([ -n "$PS1" -a -z "$HOFFMAN_SHELL_PRESERVE_PROMPT" ] && )s" +
                (isRootUser()
                    ? R"s(PS1='\n\[\033[1;31m\][hoffman-shell:\w]\$\[\033[0m\] '; )s"
                    : R"s(PS1='\n\[\033[1;32m\][hoffman-shell:\w]\$\[\033[0m\] '; )s") +
                "if [ \"$(type -t runHook)\" = function ]; then runHook shellHook; fi; "
                "unset HOFFMAN_ENFORCE_PURITY; "
                "shopt -u nullglob; "
                "unset TZ; %6%"
                "shopt -s execfail;"
                "%7%",
                escapeShellArgAlways(tmpDir.path().string()),
                (pure ? "" : "p=$PATH; "),
                (pure ? "" : "PATH=$PATH:$p; unset p; "),
                escapeShellArgAlways(std::filesystem::path(*shell).parent_path().string()),
                escapeShellArgAlways(*shell),
                tzExport,
                envCommand);
        vomit("Sourcing hoffman-shell with file %s and contents:\n%s", rcfile, rc);
        writeFile(rcfile, rc);

        Strings envStrs;
        for (auto & i : env)
            envStrs.push_back(i.first + "=" + i.second);

        auto args = interactive ? Strings{"bash", "--rcfile", rcfile} : Strings{"bash", rcfile};

        auto envPtrs = stringsToCharPtrs(envStrs);

        environ = envPtrs.data();

        auto argPtrs = stringsToCharPtrs(args);

        restoreProcessContext();

        logger->stop();

        execvp(shell->c_str(), argPtrs.data());

        throw SysError("executing shell '%s'", *shell);
    }

    else {

        std::vector<DerivedPath> pathsToBuild;
        std::vector<std::pair<StorePath, std::string>> pathsToBuildOrdered;
        RealisedPath::Set drvsToCopy;

        std::map<StorePath, std::pair<size_t, StringSet>> drvMap;

        for (auto & packageInfo : drvs) {
            auto drvPath = packageInfo.requireDrvPath();

            auto outputName = packageInfo.queryOutputName();
            if (outputName == "")
                throw Error("derivation '%s' lacks an 'outputName' attribute", store->printStorePath(drvPath));

            pathsToBuild.push_back(
                DerivedPath::Built{
                    .drvPath = makeConstantStorePathRef(drvPath),
                    .outputs = OutputsSpec::Names{outputName},
                });
            pathsToBuildOrdered.push_back({drvPath, {outputName}});
            drvsToCopy.insert(drvPath);

            auto i = drvMap.find(drvPath);
            if (i != drvMap.end())
                i->second.second.insert(outputName);
            else
                drvMap[drvPath] = {drvMap.size(), {outputName}};
        }

        buildPaths(pathsToBuild);

        if (dryRun)
            return;

        std::vector<StorePath> outPaths;

        for (auto & [drvPath, outputName] : pathsToBuildOrdered) {
            auto & [counter, _wantedOutputs] = drvMap.at({drvPath});
            std::string drvPrefix = outLink;
            if (counter)
                drvPrefix += fmt("-%d", counter + 1);

            auto outPath = deepQueryPartialDerivationOutput(*store, drvPath, outputName, &*evalStore);
            assert(outPath);
            auto outputPath = *outPath;

            if (auto store2 = store.dynamic_pointer_cast<LocalFSStore>()) {
                std::string symlink = drvPrefix;
                if (outputName != "out")
                    symlink += "-" + outputName;
                store2->addPermRoot(outputPath, absPath(symlink));
            }

            outPaths.push_back(outputPath);
        }

        logger->stop();

        for (auto & path : outPaths)
            std::cout << store->printStorePath(path) << '\n';
    }
}

static RegisterLegacyCommand r_hoffman_build("hoffman-build", main_hoffman_build);
static RegisterLegacyCommand r_hoffman_shell("hoffman-shell", main_hoffman_build);

} // namespace hoffman
