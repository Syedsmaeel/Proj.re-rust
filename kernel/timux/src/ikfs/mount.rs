//! IKFS MountTable — per-sub-kernel mount point registry
  extern crate alloc;

  use alloc::vec::Vec;
  use alloc::string::String;
  use crate::ikfs::vnode::IkfsError;

  /// A single mount point: maps a sub-kernel's local path to an IKFS vnode ID.
  #[derive(Debug, Clone)]
  pub struct MountPoint {
      pub sk_id:    u64,
      pub local:    String,   // e.g. "/mnt/shared"
      pub vnode_id: u64,
      pub writable: bool,
  }

  /// Kernel-global mount table.
  pub struct MountTable {
      entries: Vec<MountPoint>,
  }

  impl MountTable {
      pub fn new() -> Self { Self { entries: Vec::new() } }

      /// Register a mount point for a sub-kernel.
      pub fn mount(
          &mut self,
          sk_id:    u64,
          local:    impl Into<String>,
          vnode_id: u64,
          writable: bool,
      ) -> Result<(), IkfsError> {
          let local = local.into();
          // Reject duplicate (sk_id, local) pairs
          if self.entries.iter().any(|e| e.sk_id == sk_id && e.local == local) {
              return Err(IkfsError::AlreadyExists);
          }
          self.entries.push(MountPoint { sk_id, local, vnode_id, writable });
          Ok(())
      }

      /// Remove a mount point.
      pub fn umount(&mut self, sk_id: u64, local: &str) -> Result<(), IkfsError> {
          let pos = self.entries.iter().position(|e| e.sk_id == sk_id && e.local == local)
              .ok_or(IkfsError::NotFound)?;
          self.entries.remove(pos);
          Ok(())
      }

      /// Resolve a sub-kernel's local path to a vnode ID.
      pub fn resolve(&self, sk_id: u64, local: &str) -> Option<&MountPoint> {
          // Longest-prefix match
          self.entries.iter()
              .filter(|e| e.sk_id == sk_id && local.starts_with(e.local.as_str()))
              .max_by_key(|e| e.local.len())
      }

      /// All mount points for a given sub-kernel.
      pub fn for_sk(&self, sk_id: u64) -> impl Iterator<Item = &MountPoint> {
          self.entries.iter().filter(move |e| e.sk_id == sk_id)
      }

      /// Remove all mount points belonging to a terminated sub-kernel.
      pub fn evict_sk(&mut self, sk_id: u64) {
          self.entries.retain(|e| e.sk_id != sk_id);
      }

      pub fn count(&self) -> usize { self.entries.len() }
  }
  