//! re-net — Capability-gated Network Stack Sub-kernel
  //!
  //! re-net implements a lightweight userspace-style network stack that runs as
  //! a Networking sub-kernel inside Timux.  All socket operations are guarded by
  //! capability tokens minted by the master kernel.
  //!
  //! # Module layout
  //!
  //! - `socket`   — abstract socket handle + address types
  //! - `tcp`      — TCP connection state machine
  //! - `udp`      — UDP datagram endpoint
  //! - `netstack` — top-level stack that owns all sockets and dispatches packets
  //!
  //! # Capability requirements
  //!
  //! | Operation       | Required right   |
  //! |-----------------|------------------|
  //! | bind / connect  | NET_SEND         |
  //! | send            | NET_SEND         |
  //! | recv            | NET_RECV         |
  //! | close           | NET_SEND         |

  pub mod socket;
  pub mod tcp;
  pub mod udp;
  pub mod netstack;

  pub use socket::{Socket, SocketId, Addr, NetError};
  pub use tcp::{TcpSocket, TcpState};
  pub use udp::UdpSocket;
  pub use netstack::NetStack;
  