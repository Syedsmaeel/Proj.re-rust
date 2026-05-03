#pragma once
///@file

#include "hoffman/util/types.hh"
#include "hoffman/util/error.hh"
#include "hoffman/util/fun.hh"
#include "hoffman/util/logging.hh"

#include <functional>

#if defined(__FreeBSD__)
// SIGUSR1 is used by bdwgc
#  define HOFFMAN_SIG_MULTI_INT SIGTSTP
#elif !defined(_WIN32)
#  define HOFFMAN_SIG_MULTI_INT SIGUSR1
#endif

namespace hoffman {

/* User interruption. */

/**
 * @note Does nothing on Windows
 */
static inline void setInterrupted(bool isInterrupted);

/**
 * @note Does nothing on Windows
 */
static inline bool getInterrupted();

/**
 * @note Does nothing on Windows
 */
static inline bool isInterrupted();

/**
 * @note Does nothing on Windows
 */
inline void checkInterrupt();

/**
 * @note Never will happen on Windows
 */
MakeError(Interrupted, BaseError);

struct InterruptCallback
{
    virtual ~InterruptCallback() {};
};

/**
 * Register a function that gets called on SIGINT (in a non-signal
 * context).
 *
 * @note Does nothing on Windows
 */
std::unique_ptr<InterruptCallback> createInterruptCallback(fun<void()> callback);

/**
 * A RAII class that causes the current thread to receive HOFFMAN_SIG_MULTI_INT when
 * the signal handler thread receives SIGINT. That is, this allows
 * SIGINT to be multiplexed to multiple threads.
 *
 * @note Does nothing on Windows
 */
struct ReceiveInterrupts;

} // namespace hoffman

#include "hoffman/util/signals-impl.hh"
