//! Timux IPC — capability-gated message passing between rings
  //!
  //! v2 changes:
  //!  - NEW: IpcError enum replaces overloaded PrivError returns.
  //!  - BUGFIX: channel-full now returns IpcError::ChannelFull instead of
  //!    InsufficientRights (which was semantically wrong).
  //!  - NEW: configurable payload up to 4096 bytes.
  //!  - NEW: notification hooks for async wakeup (set_notify_fn).
  //!  - NEW: Channel::try_peek() for non-destructive inspection.
  //!  - NEW: MessageBuilder fluent API.
  //!  - NEW: ChannelRegistry — kernel-global channel directory.

  extern crate alloc;

  use alloc::vec::Vec;
  use alloc::collections::VecDeque;
  use crate::priv_model::{CapRight, CapabilityToken, PrivError};

  // ─── Error type ───────────────────────────────────────────────────────────────

  #[derive(Debug, Clone, Copy, PartialEq, Eq)]
  pub enum IpcError {
      InsufficientRights,
      InvalidCapability,
      /// No messages waiting in the queue
      Empty,
      /// Channel is at capacity; caller must retry
      ChannelFull,
      /// Payload exceeds MAX_PAYLOAD
      PayloadTooLarge,
      /// Target channel ID does not exist
      NoSuchChannel,
      /// Cross-ring send without the required transition capability
      RingViolation,
  }

  impl From<PrivError> for IpcError {
      fn from(e: PrivError) -> Self {
          match e {
              PrivError::InsufficientRights => IpcError::InsufficientRights,
              PrivError::InvalidCapability  => IpcError::InvalidCapability,
              _                             => IpcError::RingViolation,
          }
      }
  }

  // ─── Message ──────────────────────────────────────────────────────────────────

  /// Maximum inline payload in bytes.
  /// For large data, share a memory capability instead.
  pub const MAX_PAYLOAD: usize = 4096;

  #[derive(Debug, Clone)]
  pub struct Message {
      pub from:  u64,
      pub to:    u64,
      pub tag:   u32,
      payload:   Vec<u8>,
      pub caps:  Vec<CapabilityToken>,
  }

  impl Message {
      pub fn new(from: u64, to: u64, tag: u32, data: &[u8]) -> Result<Self, IpcError> {
          if data.len() > MAX_PAYLOAD { return Err(IpcError::PayloadTooLarge); }
          Ok(Self { from, to, tag, payload: data.to_vec(), caps: Vec::new() })
      }

      pub fn attach_cap(&mut self, cap: CapabilityToken) { self.caps.push(cap); }
      pub fn data(&self)     -> &[u8] { &self.payload }
      pub fn len(&self)      -> usize { self.payload.len() }
      pub fn is_empty(&self) -> bool  { self.payload.is_empty() }
  }

  /// Fluent builder for IPC messages.
  pub struct MessageBuilder { inner: Message }

  impl MessageBuilder {
      pub fn new(from: u64, to: u64) -> Self {
          Self { inner: Message { from, to, tag: 0, payload: Vec::new(), caps: Vec::new() } }
      }
      pub fn tag(mut self, t: u32) -> Self          { self.inner.tag = t; self }
      pub fn cap(mut self, c: CapabilityToken) -> Self { self.inner.caps.push(c); self }
      pub fn data(mut self, d: &[u8]) -> Result<Self, IpcError> {
          if d.len() > MAX_PAYLOAD { return Err(IpcError::PayloadTooLarge); }
          self.inner.payload = d.to_vec();
          Ok(self)
      }
      pub fn build(self) -> Message { self.inner }
  }

  // ─── Channel ─────────────────────────────────────────────────────────────────

  /// Callback invoked when a message is deposited into a previously-empty channel.
  pub type NotifyFn = fn(channel_id: u64);

  pub struct Channel {
      pub id:     u64,
      queue:      VecDeque<Message>,
      capacity:   usize,
      notify_fn:  Option<NotifyFn>,
      pub sends:  u64,
      pub recvs:  u64,
  }

  impl Channel {
      pub fn new(id: u64, capacity: usize) -> Self {
          Self {
              id,
              queue: VecDeque::with_capacity(capacity.min(256)),
              capacity,
              notify_fn: None,
              sends: 0,
              recvs: 0,
          }
      }

      /// Register an async wakeup callback.
      pub fn set_notify_fn(&mut self, f: NotifyFn) { self.notify_fn = Some(f); }

      /// Enqueue a message.  Requires SEND capability.
      pub fn send(&mut self, cap: &CapabilityToken, msg: Message) -> Result<(), IpcError> {
          if !cap.is_valid()              { return Err(IpcError::InvalidCapability); }
          if !cap.permits(CapRight::SEND) { return Err(IpcError::InsufficientRights); }
          if self.queue.len() >= self.capacity { return Err(IpcError::ChannelFull); }

          let was_empty = self.queue.is_empty();
          self.queue.push_back(msg);
          self.sends += 1;
          if was_empty { if let Some(f) = self.notify_fn { f(self.id); } }
          Ok(())
      }

      /// Dequeue and return a message.  Requires RECV capability.
      pub fn recv(&mut self, cap: &CapabilityToken) -> Result<Message, IpcError> {
          if !cap.is_valid()              { return Err(IpcError::InvalidCapability); }
          if !cap.permits(CapRight::RECV) { return Err(IpcError::InsufficientRights); }
          let msg = self.queue.pop_front().ok_or(IpcError::Empty)?;
          self.recvs += 1;
          Ok(msg)
      }

      /// Inspect the front message without removing it.  Requires RECV capability.
      pub fn try_peek(&self, cap: &CapabilityToken) -> Result<&Message, IpcError> {
          if !cap.is_valid()              { return Err(IpcError::InvalidCapability); }
          if !cap.permits(CapRight::RECV) { return Err(IpcError::InsufficientRights); }
          self.queue.front().ok_or(IpcError::Empty)
      }

      pub fn pending(&self) -> usize { self.queue.len() }
      pub fn is_full(&self)  -> bool  { self.queue.len() >= self.capacity }
  }

  // ─── ChannelRegistry ─────────────────────────────────────────────────────────

  /// Kernel-global channel directory — allocates IDs and owns channels.
  pub struct ChannelRegistry {
      channels: Vec<Channel>,
      next_id:  u64,
  }

  impl ChannelRegistry {
      pub fn new() -> Self { Self { channels: Vec::new(), next_id: 1 } }

      pub fn create(&mut self, capacity: usize) -> u64 {
          let id = self.next_id;
          self.next_id += 1;
          self.channels.push(Channel::new(id, capacity));
          id
      }

      pub fn get_mut(&mut self, id: u64) -> Result<&mut Channel, IpcError> {
          self.channels.iter_mut().find(|c| c.id == id).ok_or(IpcError::NoSuchChannel)
      }

      pub fn get(&self, id: u64) -> Result<&Channel, IpcError> {
          self.channels.iter().find(|c| c.id == id).ok_or(IpcError::NoSuchChannel)
      }

      pub fn destroy(&mut self, id: u64) { self.channels.retain(|c| c.id != id); }
      pub fn count(&self) -> usize { self.channels.len() }
  }
  