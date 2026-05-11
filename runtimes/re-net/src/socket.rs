//! Socket abstraction — capability-gated network endpoints
  //!
  //! A `Socket` is the public handle a sub-kernel holds.  The actual state
  //! (TCP or UDP) lives in the NetStack.

  use thiserror::Error;

  // ─── Address ─────────────────────────────────────────────────────────────────

  /// IPv4 address, host-byte-order.
  #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
  pub struct Ipv4Addr(pub u32);

  impl Ipv4Addr {
      pub const LOCALHOST: Self = Self(0x7F00_0001); // 127.0.0.1
      pub const ANY:       Self = Self(0x0000_0000); // 0.0.0.0

      pub fn octets(self) -> [u8; 4] { self.0.to_be_bytes() }

      pub fn from_octets(a: u8, b: u8, c: u8, d: u8) -> Self {
          Self(u32::from_be_bytes([a, b, c, d]))
      }
  }

  impl core::fmt::Display for Ipv4Addr {
      fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
          let [a,b,c,d] = self.octets();
          write!(f, "{a}.{b}.{c}.{d}")
      }
  }

  /// A socket address (IP + port).
  #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
  pub struct Addr {
      pub ip:   Ipv4Addr,
      pub port: u16,
  }

  impl Addr {
      pub fn new(ip: Ipv4Addr, port: u16) -> Self { Self { ip, port } }
      pub fn localhost(port: u16) -> Self { Self::new(Ipv4Addr::LOCALHOST, port) }
  }

  impl core::fmt::Display for Addr {
      fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
          write!(f, "{}:{}", self.ip, self.port)
      }
  }

  // ─── SocketId ────────────────────────────────────────────────────────────────

  pub type SocketId = u64;

  // ─── Protocol ────────────────────────────────────────────────────────────────

  #[derive(Debug, Clone, Copy, PartialEq, Eq)]
  pub enum Protocol { Tcp, Udp }

  // ─── Socket handle ───────────────────────────────────────────────────────────

  /// Public socket handle held by callers.
  #[derive(Debug, Clone)]
  pub struct Socket {
      pub id:       SocketId,
      pub protocol: Protocol,
      pub local:    Option<Addr>,
      pub remote:   Option<Addr>,
  }

  impl Socket {
      pub fn tcp(id: SocketId) -> Self { Self { id, protocol: Protocol::Tcp, local: None, remote: None } }
      pub fn udp(id: SocketId) -> Self { Self { id, protocol: Protocol::Udp, local: None, remote: None } }
      pub fn is_bound(&self)    -> bool { self.local.is_some() }
      pub fn is_connected(&self)-> bool { self.remote.is_some() }
  }

  // ─── NetError ────────────────────────────────────────────────────────────────

  #[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
  pub enum NetError {
      #[error("insufficient capability rights")]
      InsufficientRights,
      #[error("invalid capability token")]
      InvalidCapability,
      #[error("socket not found")]
      NotFound,
      #[error("address already in use")]
      AddrInUse,
      #[error("connection refused")]
      Refused,
      #[error("connection reset")]
      Reset,
      #[error("not connected")]
      NotConnected,
      #[error("already connected")]
      AlreadyConnected,
      #[error("send buffer full")]
      SendBufferFull,
      #[error("receive buffer empty")]
      RecvBufferEmpty,
      #[error("payload too large")]
      TooLarge,
      #[error("invalid state for this operation")]
      InvalidState,
      #[error("port already bound")]
      PortInUse,
  }
  