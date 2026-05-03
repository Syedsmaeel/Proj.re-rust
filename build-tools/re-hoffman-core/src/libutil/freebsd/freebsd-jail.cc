#ifdef __FreeBSD__
#  include "hoffman/util/freebsd-jail.hh"

#  include <sys/resource.h>
#  include <sys/param.h>
#  include <sys/jail.h>
#  include <sys/mount.h>

#  include "hoffman/util/error.hh"
#  include "hoffman/util/util.hh"
#  include "hoffman/util/logging.hh"

namespace hoffman {

AutoRemoveJail::AutoRemoveJail(int jid)
    : jid(jid)
{
}

void AutoRemoveJail::remove()
{
    if (jid != INVALID_JAIL) {
        if (jail_remove(jid) < 0) {
            throw SysError("Failed to remove jail %1%", jid);
        }
    }
    cancel();

    bool failed = false;
    for (auto & path : childrenMounts) {
        int r = unmount(path.c_str(), 0);
        if (r < 0 && errno == EBUSY) {
            sleep(1);
            r = unmount(path.c_str(), 0);
        }
        if (r < 0) {
            warn("Failed to unmount path %1%", PathFmt(path));
            failed = true;
        }
    }
    childrenMounts.clear();

    if (failed) {
        throw SysError("Failed to unmount some jail paths");
    }
}

AutoRemoveJail::~AutoRemoveJail()
{
    try {
        remove();
    } catch (...) {
        ignoreExceptionInDestructor();
    }
}

void AutoRemoveJail::cancel() noexcept
{
    jid = INVALID_JAIL;
}

//////////////////////////////////////////////////////////////////////

} // namespace hoffman
#endif
