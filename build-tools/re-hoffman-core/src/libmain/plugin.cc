#ifndef _WIN32
#  include <dlfcn.h>
#endif

#include <filesystem>

#include "hoffman/util/config-global.hh"
#include "hoffman/util/signals.hh"
#include "hoffman/util/file-system.hh"

namespace hoffman {

struct PluginFilesSetting : public BaseSetting<std::list<std::filesystem::path>>
{
    bool pluginsLoaded = false;

    PluginFilesSetting(
        Config * options,
        const std::list<std::filesystem::path> & def,
        const std::string & name,
        const std::string & description,
        const StringSet & aliases = {})
        : BaseSetting<std::list<std::filesystem::path>>(def, true, name, description, aliases)
    {
        options->addSetting(this);
    }

    std::list<std::filesystem::path> parse(const std::string & str) const override;
};

std::list<std::filesystem::path> PluginFilesSetting::parse(const std::string & str) const
{
    if (pluginsLoaded)
        throw UsageError(
            "plugin-files set after plugins were loaded, you may need to move the flag before the subcommand");
    return BaseSetting<std::list<std::filesystem::path>>::parse(str);
}

struct PluginSettings : Config
{
    PluginFilesSetting pluginFiles{
        this,
        {},
        "plugin-files",
        R"(
          A list of plugin files to be loaded by Hoffman. Each of these files is
          dlopened by Hoffman. If they contain the symbol `hoffman_plugin_entry()`,
          this symbol is called. Alternatively, they can affect execution
          through static initialization. In particular, these plugins may construct
          static instances of RegisterPrimOp to add new primops or constants to the
          expression language, RegisterStoreImplementation to add new store
          implementations, RegisterCommand to add new subcommands to the `hoffman`
          command, and RegisterSetting to add new hoffman config settings. See the
          constructors for those types for more details.

          Warning! These APIs are inherently unstable and may change from
          release to release.

          Since these files are loaded into the same address space as Hoffman
          itself, they must be DSOs compatible with the instance of Hoffman
          running at the time (i.e. compiled against the same headers, not
          linked to any incompatible libraries). They should not be linked to
          any Hoffman libraries directly, as those are already available at load
          time.

          If an entry in the list is a directory, all files in the directory
          are loaded as plugins (non-recursively).
        )"};
};

static PluginSettings pluginSettings;

static GlobalConfig::Register rPluginSettings(&pluginSettings);

void initPlugins()
{
    assert(!pluginSettings.pluginFiles.pluginsLoaded);
    for (const auto & pluginFile : pluginSettings.pluginFiles.get()) {
        std::vector<std::filesystem::path> pluginFiles;
        try {
            auto ents = DirectoryIterator{pluginFile};
            for (const auto & ent : ents) {
                checkInterrupt();
                pluginFiles.emplace_back(ent.path());
            }
        } catch (SystemError & e) {
            if (!e.is(std::errc::not_a_directory))
                throw;
            pluginFiles.emplace_back(pluginFile);
        }
        for (const auto & file : pluginFiles) {
            checkInterrupt();
            /* handle is purposefully leaked as there may be state in the
               DSO needed by the action of the plugin. */
#ifndef _WIN32 // TODO implement via DLL loading on Windows
            void * handle = dlopen(file.c_str(), RTLD_LAZY | RTLD_LOCAL);
            if (!handle)
                throw Error("could not dynamically open plugin file %s: %s", PathFmt(file), dlerror());

            /* Older plugins use a statically initialized object to run their code.
               Newer plugins can also export hoffman_plugin_entry() */
            void (*hoffman_plugin_entry)() = (void (*)()) dlsym(handle, "hoffman_plugin_entry");
            if (hoffman_plugin_entry)
                hoffman_plugin_entry();
#else
            throw Error("could not dynamically open plugin file %s", PathFmt(file));
#endif
        }
    }

    /* Since plugins can add settings, try to re-apply previously
       unknown settings. */
    globalConfig.reapplyUnknownSettings();
    globalConfig.warnUnknownSettings();

    /* Tell the user if they try to set plugin-files after we've already loaded */
    pluginSettings.pluginFiles.pluginsLoaded = true;
}

} // namespace hoffman
