//! TCP connection state machine
  //!
  //! Implements a simplified TCP finite state machine sufficient for
  //! single-connection sub-kernel communication within the Re-Rust ecosystem.
  //!
  //! Supported states: Closed → Listen → SynReceived → Established →
  //!                   FinWait1 → FinWait2 → TimeWait → Closed
  //!                   Closed → SynSent → Established (active open)

  use std::collections::VecDeque;
  use crate::socket::{Addr, NetError, SocketId};

  // ─── TCP state ───────────────────────────────────────────────────────────────

  #[derive(Debug, Clone, Copy, PartialEq, Eq)]
  pub enum TcpState {
      Closed,
      Listen,
      SynSent,
      SynReceived,
      Established,
      FinWait1,
      FinWait2,
      TimeWait,
      CloseWait,
      LastAck,
  }

  impl TcpState {
      pub fn is_open(self) -> bool {
          matches!(self, Self::Established | Self::CloseWait)
      }
      pub fn name(self) -> &'static str {
          match self {
              Self::Closed      => "CLOSED",
              Self::Listen      => "LISTEN",
              Self::SynSent     => "SYN_SENT",
              Self::SynReceived => "SYN_RECEIVED",
              Self::Established => "ESTABLISHED",
              Self::FinWait1    => "FIN_WAIT_1",
              Self::FinWait2    => "FIN_WAIT_2",
              Self::TimeWait    => "TIME_WAIT",
              Self::CloseWait   => "CLOSE_WAIT",
              Self::LastAck     => "LAST_ACK",
          }
      }
  }

  // ─── Sequence tracking ───────────────────────────────────────────────────────

  #[derive(Debug, Clone, Default)]
  pub struct SeqTracker {
      pub send_next: u32,
      pub recv_next: u32,
      pub send_una:  u32,
      pub window:    u16,
  }

  // ─── TcpSocket ───────────────────────────────────────────────────────────────

  pub const TCP_BUF: usize = 65536; // 64 KiB rx/tx buffers

  pub struct TcpSocket {
      pub id:     SocketId,
      pub state:  TcpState,
      pub local:  Option<Addr>,
      pub remote: Option<Addr>,
      pub seq:    SeqTracker,
      tx_buf:     VecDeque<u8>,
      rx_buf:     VecDeque<u8>,
      pub bytes_sent: u64,
      pub bytes_recv: u64,
  }

  impl TcpSocket {
      pub fn new(id: SocketId) -> Self {
          Self {
              id,
              state:      TcpState::Closed,
              local:      None,
              remote:     None,
              seq:        SeqTracker::default(),
              tx_buf:     VecDeque::with_capacity(TCP_BUF),
              rx_buf:     VecDeque::with_capacity(TCP_BUF),
              bytes_sent: 0,
              bytes_recv: 0,
          }
      }

      // ── Active open ──────────────────────────────────────────────────────────

      /// Initiate connection to a remote address (SYN sent).
      pub fn connect(&mut self, local: Addr, remote: Addr, isn: u32) -> Result<(), NetError> {
          if self.state != TcpState::Closed { return Err(NetError::InvalidState); }
          self.local  = Some(local);
          self.remote = Some(remote);
          self.seq.send_next = isn.wrapping_add(1);
          self.seq.send_una  = isn;
          self.state = TcpState::SynSent;
          Ok(())
      }

      /// Process an incoming SYN-ACK (move to ESTABLISHED).
      pub fn on_syn_ack(&mut self, their_isn: u32) -> Result<(), NetError> {
          if self.state != TcpState::SynSent { return Err(NetError::InvalidState); }
          self.seq.recv_next = their_isn.wrapping_add(1);
          self.state = TcpState::Established;
          Ok(())
      }

      // ── Passive open ─────────────────────────────────────────────────────────

      /// Begin listening for incoming connections.
      pub fn listen(&mut self, local: Addr) -> Result<(), NetError> {
          if self.state != TcpState::Closed { return Err(NetError::InvalidState); }
          self.local = Some(local);
          self.state = TcpState::Listen;
          Ok(())
      }

      /// Process an incoming SYN (move to SYN_RECEIVED, send SYN-ACK).
      pub fn on_syn(&mut self, remote: Addr, their_isn: u32, our_isn: u32) -> Result<(), NetError> {
          if self.state != TcpState::Listen { return Err(NetError::InvalidState); }
          self.remote        = Some(remote);
          self.seq.recv_next = their_isn.wrapping_add(1);
          self.seq.send_next = our_isn.wrapping_add(1);
          self.seq.send_una  = our_isn;
          self.state = TcpState::SynReceived;
          Ok(())
      }

      /// Process the final ACK of the three-way handshake.
      pub fn on_ack(&mut self, ack_num: u32) -> Result<(), NetError> {
          match self.state {
              TcpState::SynReceived => {
                  self.seq.send_una = ack_num;
                  self.state = TcpState::Established;
                  Ok(())
              }
              TcpState::FinWait1 => {
                  self.state = TcpState::FinWait2;
                  Ok(())
              }
              TcpState::LastAck => {
                  self.state = TcpState::Closed;
                  Ok(())
              }
              _ => Ok(()), // ignore spurious ACKs
          }
      }

      // ── Data transfer ────────────────────────────────────────────────────────

      /// Enqueue data to send.
      pub fn send(&mut self, data: &[u8]) -> Result<usize, NetError> {
          if !self.state.is_open()        { return Err(NetError::NotConnected); }
          let space = TCP_BUF.saturating_sub(self.tx_buf.len());
          if space == 0                   { return Err(NetError::SendBufferFull); }
          let n = data.len().min(space);
          self.tx_buf.extend(&data[..n]);
          self.bytes_sent += n as u64;
          Ok(n)
      }

      /// Simulate receiving data from the network into the rx buffer.
      pub fn on_data(&mut self, data: &[u8]) -> Result<(), NetError> {
          if self.state != TcpState::Established { return Err(NetError::NotConnected); }
          let space = TCP_BUF.saturating_sub(self.rx_buf.len());
          let n = data.len().min(space);
          self.rx_buf.extend(&data[..n]);
          self.bytes_recv += n as u64;
          Ok(())
      }

      /// Read from the rx buffer.
      pub fn recv(&mut self, buf: &mut [u8]) -> Result<usize, NetError> {
          if self.rx_buf.is_empty() { return Err(NetError::RecvBufferEmpty); }
          let n = self.rx_buf.len().min(buf.len());
          for (i, b) in buf[..n].iter_mut().enumerate() {
              *b = self.rx_buf[i];
          }
          self.rx_buf.drain(..n);
          Ok(n)
      }

      /// Drain the tx buffer (simulates the NIC consuming bytes).
      pub fn drain_tx(&mut self, max: usize) -> Vec<u8> {
          let n = self.tx_buf.len().min(max);
          self.tx_buf.drain(..n).collect()
      }

      // ── Close ────────────────────────────────────────────────────────────────

      /// Initiate graceful close (active FIN).
      pub fn close(&mut self) -> Result<(), NetError> {
          match self.state {
              TcpState::Established => { self.state = TcpState::FinWait1; Ok(()) }
              TcpState::CloseWait   => { self.state = TcpState::LastAck;  Ok(()) }
              _                     => Err(NetError::InvalidState),
          }
      }

      /// Process incoming FIN.
      pub fn on_fin(&mut self) -> Result<(), NetError> {
          match self.state {
              TcpState::Established => { self.state = TcpState::CloseWait; Ok(()) }
              TcpState::FinWait2    => { self.state = TcpState::TimeWait;  Ok(()) }
              _                     => Err(NetError::InvalidState),
          }
      }

      pub fn tx_pending(&self) -> usize { self.tx_buf.len() }
      pub fn rx_available(&self) -> usize { self.rx_buf.len() }
  }
  