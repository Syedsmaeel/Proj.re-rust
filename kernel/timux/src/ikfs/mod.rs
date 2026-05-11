//! IKFS — Inter-Kernel Filesystem
  //!
  //! A shared virtual filesystem that sub-kernels can mount and use to
  //! communicate through files, directories, and IPC pipes.  All access
  //! is capability-gated via the Timux privilege model.
  //!
  //! # Architecture
  //!
  //! ```text
  //! ┌─────────────────────────────────────────────────────┐
  //! │                  Sub-kernel A                       │
  //! │   open("/ikfs/shared/config") → cap(READ) ──────┐  │
  //! └────────────────────────────────────────────┬────│──┘
  //!                                              │    │
  //!              IKFS Namespace                  │    │
  //!  ┌──────────────────────────────────────┐   │    │
  //!  │  /ikfs/                              │   │    │
  //!  │    shared/   (world-readable dir)    │   │    │
  //!  │    pipes/    (IPC pipe endpoints)    │   │    │
  //!  │    priv/     (privileged, ring ≤ 1)  │   │    │
  //!  └──────────────────────────────────────┘   │    │
  //!                                              │    │
  //! ┌───────────────────────────────────────────▼────▼──┐
  //! │                  Sub-kernel B                      │
  //! │   write("/ikfs/pipes/sk-a-b") → IPC message       │
  //! └────────────────────────────────────────────────────┘
  //! ```

  #![allow(dead_code)]
  extern crate alloc;

  pub mod vnode;
  pub mod mount;

  pub use vnode::{Vnode, VnodeKind, VnodeId, IkfsError};
  pub use mount::{MountTable, MountPoint};

  use alloc::vec::Vec;
  use alloc::string::String;
  use crate::priv_model::{CapRight, CapabilityToken, RingLevel};

  // ─── IKFS root ───────────────────────────────────────────────────────────────

  /// The global IKFS instance.
  pub struct Ikfs {
      pub mounts: MountTable,
      vnodes:     Vec<Vnode>,
      next_id:    VnodeId,
  }

  impl Ikfs {
      pub fn new() -> Self {
          let mut fs = Self { mounts: MountTable::new(), vnodes: Vec::new(), next_id: 1 };
          // Provision the standard top-level directories
          fs.create_dir_unchecked("shared");
          fs.create_dir_unchecked("pipes");
          fs.create_dir_unchecked("priv");
          fs
      }

      // ── Internal helpers ─────────────────────────────────────────────────────

      fn alloc_id(&mut self) -> VnodeId {
          let id = self.next_id;
          self.next_id += 1;
          id
      }

      fn create_dir_unchecked(&mut self, name: &str) -> VnodeId {
          let id = self.alloc_id();
          self.vnodes.push(Vnode::dir(id, name));
          id
      }

      fn find(&self, name: &str) -> Option<&Vnode> {
          self.vnodes.iter().find(|v| v.name == name)
      }

      fn find_mut(&mut self, name: &str) -> Option<&mut Vnode> {
          self.vnodes.iter_mut().find(|v| v.name == name)
      }

      fn find_by_id(&self, id: VnodeId) -> Option<&Vnode> {
          self.vnodes.iter().find(|v| v.id == id)
      }

      // ── Public API ───────────────────────────────────────────────────────────

      /// Create a file in the shared namespace.  Requires FS_WRITE capability.
      pub fn create_file(
          &mut self,
          cap: &CapabilityToken,
          name: &str,
          owner_sk: u64,
      ) -> Result<VnodeId, IkfsError> {
          if !cap.is_valid()                { return Err(IkfsError::NotPermitted); }
          if !cap.permits(CapRight::FS_WRITE) { return Err(IkfsError::NotPermitted); }
          if self.find(name).is_some()        { return Err(IkfsError::AlreadyExists); }
          let id = self.alloc_id();
          self.vnodes.push(Vnode::file(id, name, owner_sk));
          Ok(id)
      }

      /// Create a bidirectional IPC pipe.  Requires SEND | RECV capability.
      pub fn create_pipe(
          &mut self,
          cap: &CapabilityToken,
          name: &str,
          sk_a: u64,
          sk_b: u64,
      ) -> Result<VnodeId, IkfsError> {
          if !cap.is_valid()               { return Err(IkfsError::NotPermitted); }
          if !cap.permits(CapRight::SEND)  { return Err(IkfsError::NotPermitted); }
          if self.find(name).is_some()     { return Err(IkfsError::AlreadyExists); }
          let id = self.alloc_id();
          self.vnodes.push(Vnode::pipe(id, name, sk_a, sk_b));
          Ok(id)
      }

      /// Write data to a file vnode.  Requires FS_WRITE capability.
      pub fn write(
          &mut self,
          cap: &CapabilityToken,
          id: VnodeId,
          data: &[u8],
      ) -> Result<usize, IkfsError> {
          if !cap.is_valid()                { return Err(IkfsError::NotPermitted); }
          if !cap.permits(CapRight::FS_WRITE) { return Err(IkfsError::NotPermitted); }
          let vnode = self.vnodes.iter_mut().find(|v| v.id == id)
              .ok_or(IkfsError::NotFound)?;
          vnode.write(data)
      }

      /// Read data from a file vnode.  Requires FS_READ capability.
      pub fn read(
          &self,
          cap: &CapabilityToken,
          id: VnodeId,
          offset: usize,
          buf: &mut [u8],
      ) -> Result<usize, IkfsError> {
          if !cap.is_valid()               { return Err(IkfsError::NotPermitted); }
          if !cap.permits(CapRight::FS_READ) { return Err(IkfsError::NotPermitted); }
          let vnode = self.vnodes.iter().find(|v| v.id == id)
              .ok_or(IkfsError::NotFound)?;
          vnode.read(offset, buf)
      }

      /// Unlink (remove) a vnode.  Requires FS_WRITE capability.
      pub fn unlink(&mut self, cap: &CapabilityToken, id: VnodeId) -> Result<(), IkfsError> {
          if !cap.is_valid()                { return Err(IkfsError::NotPermitted); }
          if !cap.permits(CapRight::FS_WRITE) { return Err(IkfsError::NotPermitted); }
          let pos = self.vnodes.iter().position(|v| v.id == id)
              .ok_or(IkfsError::NotFound)?;
          self.vnodes.remove(pos);
          Ok(())
      }

      /// List all vnode names.
      pub fn list(&self, cap: &CapabilityToken) -> Result<Vec<&str>, IkfsError> {
          if !cap.is_valid()               { return Err(IkfsError::NotPermitted); }
          if !cap.permits(CapRight::FS_READ) { return Err(IkfsError::NotPermitted); }
          Ok(self.vnodes.iter().map(|v| v.name.as_str()).collect())
      }

      pub fn vnode_count(&self) -> usize { self.vnodes.len() }
  }
  