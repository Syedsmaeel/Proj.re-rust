//! NetStack — top-level network stack that owns all sockets
  //!
  //! The NetStack runs as a Networking sub-kernel service.  The master kernel
  //! capability-gates every operation through NET_SEND / NET_RECV rights.
  //!
  //! Socket IDs are allocated monotonically and never reused within a session.

  use std::collections::HashMap;
  use crate::socket::{Addr, NetError, Protocol, Socket, SocketId};
  use crate::tcp::TcpSocket;
  use crate::udp::UdpSocket;

  // Simulating capability checking — in no_std kernel use CapabilityToken directly.
  // Here we use a simple boolean flag to keep the crate std-compatible.
  pub struct NetCap {
      pub can_send: bool,
      pub can_recv: bool,
  }

  impl NetCap {
      pub fn full()  -> Self { Self { can_send: true,  can_recv: true } }
      pub fn recv()  -> Self { Self { can_send: false, can_recv: true } }
      pub fn send()  -> Self { Self { can_send: true,  can_recv: false } }
      pub fn check_send(&self) -> Result<(), NetError> {
          if self.can_send { Ok(()) } else { Err(NetError::InsufficientRights) }
      }
      pub fn check_recv(&self) -> Result<(), NetError> {
          if self.can_recv { Ok(()) } else { Err(NetError::InsufficientRights) }
      }
  }

  // ─── Port registry ───────────────────────────────────────────────────────────

  struct PortSet(std::collections::HashSet<u16>);
  impl PortSet {
      fn new() -> Self { Self(std::collections::HashSet::new()) }
      fn claim(&mut self, port: u16) -> Result<(), NetError> {
          if !self.0.insert(port) { Err(NetError::PortInUse) } else { Ok(()) }
      }
      fn release(&mut self, port: u16) { self.0.remove(&port); }
  }

  // ─── NetStack ────────────────────────────────────────────────────────────────

  pub struct NetStack {
      tcp:      HashMap<SocketId, TcpSocket>,
      udp:      HashMap<SocketId, UdpSocket>,
      ports:    PortSet,
      next_id:  SocketId,
  }

  impl NetStack {
      pub fn new() -> Self {
          Self { tcp: HashMap::new(), udp: HashMap::new(), ports: PortSet::new(), next_id: 1 }
      }

      fn alloc_id(&mut self) -> SocketId { let id = self.next_id; self.next_id += 1; id }

      // ── Socket lifecycle ─────────────────────────────────────────────────────

      /// Create a new TCP socket.  Requires send capability.
      pub fn tcp_open(&mut self, cap: &NetCap) -> Result<Socket, NetError> {
          cap.check_send()?;
          let id = self.alloc_id();
          self.tcp.insert(id, TcpSocket::new(id));
          Ok(Socket::tcp(id))
      }

      /// Create a new UDP socket.  Requires send capability.
      pub fn udp_open(&mut self, cap: &NetCap) -> Result<Socket, NetError> {
          cap.check_send()?;
          let id = self.alloc_id();
          self.udp.insert(id, UdpSocket::new(id));
          Ok(Socket::udp(id))
      }

      /// Close and remove a socket.
      pub fn close(&mut self, cap: &NetCap, id: SocketId) -> Result<(), NetError> {
          cap.check_send()?;
          if let Some(tcp) = self.tcp.remove(&id) {
              if let Some(local) = tcp.local { self.ports.release(local.port); }
              return Ok(());
          }
          if let Some(udp) = self.udp.remove(&id) {
              if let Some(local) = udp.local { self.ports.release(local.port); }
              return Ok(());
          }
          Err(NetError::NotFound)
      }

      // ── TCP operations ───────────────────────────────────────────────────────

      /// Connect a TCP socket to a remote address.
      pub fn tcp_connect(
          &mut self, cap: &NetCap, id: SocketId,
          local: Addr, remote: Addr, isn: u32,
      ) -> Result<(), NetError> {
          cap.check_send()?;
          self.ports.claim(local.port)?;
          let sock = self.tcp.get_mut(&id).ok_or(NetError::NotFound)?;
          sock.connect(local, remote, isn)?;
          Ok(())
      }

      /// Start listening on a TCP socket.
      pub fn tcp_listen(&mut self, cap: &NetCap, id: SocketId, local: Addr) -> Result<(), NetError> {
          cap.check_send()?;
          self.ports.claim(local.port)?;
          let sock = self.tcp.get_mut(&id).ok_or(NetError::NotFound)?;
          sock.listen(local)?;
          Ok(())
      }

      /// Send data over TCP.
      pub fn tcp_send(&mut self, cap: &NetCap, id: SocketId, data: &[u8]) -> Result<usize, NetError> {
          cap.check_send()?;
          let sock = self.tcp.get_mut(&id).ok_or(NetError::NotFound)?;
          sock.send(data)
      }

      /// Receive data from TCP rx buffer.
      pub fn tcp_recv(&mut self, cap: &NetCap, id: SocketId, buf: &mut [u8]) -> Result<usize, NetError> {
          cap.check_recv()?;
          let sock = self.tcp.get_mut(&id).ok_or(NetError::NotFound)?;
          sock.recv(buf)
      }

      /// Close a TCP connection gracefully.
      pub fn tcp_close(&mut self, cap: &NetCap, id: SocketId) -> Result<(), NetError> {
          cap.check_send()?;
          let sock = self.tcp.get_mut(&id).ok_or(NetError::NotFound)?;
          sock.close()
      }

      // ── UDP operations ───────────────────────────────────────────────────────

      /// Bind a UDP socket to a local address.
      pub fn udp_bind(&mut self, cap: &NetCap, id: SocketId, local: Addr) -> Result<(), NetError> {
          cap.check_send()?;
          self.ports.claim(local.port)?;
          let sock = self.udp.get_mut(&id).ok_or(NetError::NotFound)?;
          sock.bind(local)?;
          Ok(())
      }

      /// Send a UDP datagram.
      pub fn udp_send(&mut self, cap: &NetCap, id: SocketId, data: &[u8], remote: Addr) -> Result<usize, NetError> {
          cap.check_send()?;
          let sock = self.udp.get_mut(&id).ok_or(NetError::NotFound)?;
          sock.send_to(data, remote)
      }

      /// Receive a UDP datagram.
      pub fn udp_recv(&mut self, cap: &NetCap, id: SocketId) -> Result<crate::udp::Datagram, NetError> {
          cap.check_recv()?;
          let sock = self.udp.get_mut(&id).ok_or(NetError::NotFound)?;
          sock.recv_from()
      }

      // ── Stats ────────────────────────────────────────────────────────────────

      pub fn tcp_count(&self) -> usize { self.tcp.len() }
      pub fn udp_count(&self) -> usize { self.udp.len() }
      pub fn socket_count(&self) -> usize { self.tcp.len() + self.udp.len() }

      /// TCP socket state for monitoring.
      pub fn tcp_state(&self, id: SocketId) -> Option<crate::tcp::TcpState> {
          self.tcp.get(&id).map(|s| s.state)
      }
  }
  