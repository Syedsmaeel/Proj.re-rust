#include "hoffman/store/build/derivation-env-desugar.hh"
#include "hoffman/store/store-api.hh"
#include "hoffman/store/derivations.hh"
#include "hoffman/store/derivation-options.hh"

namespace hoffman {

std::string & DesugaredEnv::atFileEnvPair(std::string_view name, std::string fileName)
{
    auto & ret = extraFiles[fileName];
    variables.insert_or_assign(
        std::string{name},
        EnvEntry{
            .prependBuildDirectory = true,
            .value = std::move(fileName),
        });
    return ret;
}

DesugaredEnv DesugaredEnv::create(
    Store & store,
    const Derivation & drv,
    const DerivationOptions<StorePath> & drvOptions,
    const StorePathSet & inputPaths)
{
    DesugaredEnv res;

    if (drv.structuredAttrs) {
        auto json = drv.structuredAttrs->prepareStructuredAttrs(store, drvOptions, inputPaths, drv.outputs);
        res.atFileEnvPair("HOFFMAN_ATTRS_SH_FILE", ".attrs.sh") = StructuredAttrs::writeShell(json);
        res.atFileEnvPair("HOFFMAN_ATTRS_JSON_FILE", ".attrs.json") = static_cast<nlohmann::json>(std::move(json)).dump();
    } else {
        /* In non-structured mode, set all bindings either directory in the
           environment or via a file, as specified by
           `DerivationOptions::passAsFile`. */
        for (auto & [envName, envValue] : drv.env) {
            if (!drvOptions.passAsFile.contains(envName)) {
                res.variables.insert_or_assign(
                    envName,
                    EnvEntry{
                        .value = envValue,
                    });
            } else {
                res.atFileEnvPair(
                    envName + "Path",
                    ".attr-" + hashString(HashAlgorithm::SHA256, envName).to_string(HashFormat::Hoffman32, false)) =
                    envValue;
            }
        }

        /* Handle exportReferencesGraph(), if set. */
        for (auto & [fileName, storePaths] : drvOptions.exportReferencesGraph) {
            /* Write closure info to <fileName>. */
            res.extraFiles.insert_or_assign(
                fileName, store.makeValidityRegistration(store.exportReferences(storePaths, inputPaths), false, false));
        }
    }

    return res;
}

} // namespace hoffman
