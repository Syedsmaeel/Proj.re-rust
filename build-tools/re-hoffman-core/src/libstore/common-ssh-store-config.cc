#include "hoffman/store/common-ssh-store-config.hh"
#include "hoffman/store/ssh.hh"

namespace hoffman {

CommonSSHStoreConfig::CommonSSHStoreConfig(const ParsedURL::Authority & authority, const Params & params)
    : StoreConfig(params, FilePathType::Uhoffman)
    , authority(authority)
{
}

void CommonSSHStoreConfig::anchor() {}

SSHMaster CommonSSHStoreConfig::createSSHMaster(bool useMaster, Descriptor logFD) const
{
    return {
        authority,
        sshKey.get(),
        sshPublicHostKey.get(),
        useMaster,
        compress,
        logFD,
    };
}

} // namespace hoffman
