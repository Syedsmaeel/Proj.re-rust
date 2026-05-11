//! UDP datagram endpoint
  //!
  //! UDP in re-net is stateless at the protocol level.  Each UdpSocket binds
  //! to a local address and can send/receive datagrams to/from any remote.
  //!
  //! Datagrams are queued in an in-kernel ring buffer.  Each datagram carries
  //! its source address so the application can reply.

  use std::collections::VecDeque;
  use crate::socket::{Addr, NetError, SocketId};

  // ─── Datagram ────────────────────────────────────────────────────────────────

  /// Maximum UDP payload (bytes).
  pub const MAX_DATAGRAM: usize = 65507;

  /// A received datagram with its origin address.
  #[derive(Debug, Clone)]
  pub struct Datagram {
      pub from:    Addr,
      pub payload: Vec<u8>,
  }

  // ─── UdpSocket ───────────────────────────────────────────────────────────────

  pub struct UdpSocket {
      pub id:      SocketId,
      pub local:   Option<Addr>,
      rx_queue:    VecDeque<Datagram>,
      capacity:    usize,
      pub tx_pkts: u64,
      pub rx_pkts: u64,
      pub tx_bytes:u64,
      pub rx_bytes:u64,
  }

  impl UdpSocket {
      pub fn new(id: SocketId) -> Self {
          Self {
              id,
              local:    None,
              rx_queue: VecDeque::new(),
              capacity: 256,
              tx_pkts: 0, rx_pkts: 0, tx_bytes: 0, rx_bytes: 0,
          }
      }

      /// Bind to a local address.
      pub fn bind(&mut self, local: Addr) -> Result<(), NetError> {
          if self.local.is_some() { return Err(NetError::AlreadyConnected); }
          self.local = Some(local);
          Ok(())
      }

      /// Enqueue data to send to `remote`.
      /// In a real stack this would hand the datagram to the IP layer.
      /// Here we record the transmission stats and return the bytes "sent".
      pub fn send_to(&mut self, data: &[u8], _remote: Addr) -> Result<usize, NetError> {
          if self.local.is_none()        { return Err(NetError::NotConnected); }
          if data.len() > MAX_DATAGRAM   { return Err(NetError::TooLarge); }
          self.tx_pkts  += 1;
          self.tx_bytes += data.len() as u64;
          Ok(data.len())
      }

      /// Simulate receiving a datagram from the network.
      pub fn on_datagram(&mut self, from: Addr, payload: Vec<u8>) -> Result<(), NetError> {
          if payload.len() > MAX_DATAGRAM { return Err(NetError::TooLarge); }
          if self.rx_queue.len() >= self.capacity {
              // Drop tail — oldest-first drop policy
              self.rx_queue.pop_front();
          }
          self.rx_pkts  += 1;
          self.rx_bytes += payload.len() as u64;
          self.rx_queue.push_back(Datagram { from, payload });
          Ok(())
      }

      /// Receive the next datagram.
      pub fn recv_from(&mut self) -> Result<Datagram, NetError> {
          self.rx_queue.pop_front().ok_or(NetError::RecvBufferEmpty)
      }

      /// Peek at the next datagram without removing it.
      pub fn peek(&self) -> Option<&Datagram> { self.rx_queue.front() }

      pub fn is_bound(&self)   -> bool  { self.local.is_some() }
      pub fn pending(&self)    -> usize { self.rx_queue.len() }
  }
  