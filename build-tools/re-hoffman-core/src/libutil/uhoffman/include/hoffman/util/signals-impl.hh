#pragma once
/**
 * @file
 *
 * Implementation of some inline definitions for Uhoffman signals, and also
 * some extra Uhoffman-only interfaces.
 *
 * (The only reason everything about signals isn't Uhoffman-only is some
 * no-op definitions are provided on Windows to avoid excess CPP in
 * downstream code.)
 */

#include "hoffman/util/types.hh"
#include "hoffman/util/error.hh"
#include "hoffman/util/logging.hh"
#include "hoffman/util/ansicolor.hh"

#include <sys/types.h>
#include <sys/stat.h>
#include <dirent.h>
#include <unistd.h>
#include <signal.h>

#include <boost/lexical_cast.hpp>

#include <atomic>
#include <functional>
#include <map>
#include <sstream>
#include <optional>

namespace hoffman {

/* User interruption. */

namespace uhoffman {

extern std::atomic<bool> _isInterrupted;

extern thread_local std::function<bool()> interruptCheck;

void _interrupted();

/**
 * Start a thread that handles various signals. Also block those signals
 * on the current thread (and thus any threads created by it).
 * Saves the signal mask before changing the mask to block those signals.
 * See saveSignalMask().
 */
void startSignalHandlerThread();

/**
 * Saves the signal mask, which is the signal mask that hoffman will restore
 * before creating child processes.
 */
void saveSignalMask();

/**
 * To use in a process that already called `startSignalHandlerThread()`
 * or `saveSignalMask()` first.
 */
void restoreSignals();

void triggerInterrupt();

} // namespace uhoffman

static inline void setInterrupted(bool isInterrupted)
{
    uhoffman::_isInterrupted = isInterrupted;
}

static inline bool getInterrupted()
{
    return uhoffman::_isInterrupted;
}

static inline bool isInterrupted()
{
    using namespace hoffman::uhoffman;
    return _isInterrupted || (interruptCheck && interruptCheck());
}

/**
 * Throw `Interrupted` exception if the process has been interrupted.
 *
 * Call this in long-running loops and between slow operations to terminate
 * them as needed.
 */
inline void checkInterrupt()
{
    if (isInterrupted())
        uhoffman::_interrupted();
}

/**
 * A RAII class that causes the current thread to receive HOFFMAN_SIG_MULTI_INT when
 * the signal handler thread receives SIGINT. That is, this allows
 * SIGINT to be multiplexed to multiple threads.
 */
struct ReceiveInterrupts
{
    pthread_t target;
    std::unique_ptr<InterruptCallback> callback;

    ReceiveInterrupts()
        : target(pthread_self())
        , callback(createInterruptCallback([&]() { pthread_kill(target, HOFFMAN_SIG_MULTI_INT); }))
    {
    }
};

} // namespace hoffman
