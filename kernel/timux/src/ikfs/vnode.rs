//! IKFS Vnode — virtual filesystem node types
  extern crate alloc;

  use alloc::vec::Vec;
  use alloc::string::String;

  pub type VnodeId = u64;

  // ─── Error ───────────────────────────────────────────────────────────────────

  #[derive(Debug, Clone, Copy, PartialEq, Eq)]
  pub enum IkfsError {
      NotFound,
      AlreadyExists,
      NotPermitted,
      IsDirectory,
      NotDirectory,
      NoSpace,
      InvalidOffset,
      BrokenPipe,
  }

  // ─── VnodeKind ───────────────────────────────────────────────────────────────

  #[derive(Debug, Clone)]
  pub enum VnodeKind {
      /// A regular byte-stream file backed by an in-kernel buffer.
      File {
          data:     Vec<u8>,
          owner_sk: u64,
      },
      /// A directory (logical grouping only in flat IKFS namespace).
      Directory,
      /// A bidirectional IPC pipe between two sub-kernels.
      Pipe {
          sk_a:    u64,
          sk_b:    u64,
          a_to_b:  Vec<u8>,
          b_to_a:  Vec<u8>,
          max_buf: usize,
      },
  }

  // ─── Vnode ───────────────────────────────────────────────────────────────────

  #[derive(Debug, Clone)]
  pub struct Vnode {
      pub id:         VnodeId,
      pub name:       String,
      pub kind:       VnodeKind,
      pub read_count: u64,
      pub write_count:u64,
  }

  impl Vnode {
      pub fn file(id: VnodeId, name: &str, owner_sk: u64) -> Self {
          Self {
              id, name: name.into(),
              kind: VnodeKind::File { data: Vec::new(), owner_sk },
              read_count: 0, write_count: 0,
          }
      }

      pub fn dir(id: VnodeId, name: &str) -> Self {
          Self { id, name: name.into(), kind: VnodeKind::Directory, read_count: 0, write_count: 0 }
      }

      pub fn pipe(id: VnodeId, name: &str, sk_a: u64, sk_b: u64) -> Self {
          Self {
              id, name: name.into(),
              kind: VnodeKind::Pipe { sk_a, sk_b, a_to_b: Vec::new(), b_to_a: Vec::new(), max_buf: 65536 },
              read_count: 0, write_count: 0,
          }
      }

      pub fn is_file(&self) -> bool  { matches!(&self.kind, VnodeKind::File { .. }) }
      pub fn is_dir(&self) -> bool   { matches!(&self.kind, VnodeKind::Directory) }
      pub fn is_pipe(&self) -> bool  { matches!(&self.kind, VnodeKind::Pipe { .. }) }

      /// Write bytes to a file vnode.
      pub fn write(&mut self, data: &[u8]) -> Result<usize, IkfsError> {
          match &mut self.kind {
              VnodeKind::File { data: buf, .. } => {
                  buf.extend_from_slice(data);
                  self.write_count += 1;
                  Ok(data.len())
              }
              VnodeKind::Directory => Err(IkfsError::IsDirectory),
              VnodeKind::Pipe { .. } => Err(IkfsError::BrokenPipe),
          }
      }

      /// Read bytes from a file vnode at the given offset.
      pub fn read(&self, offset: usize, buf: &mut [u8]) -> Result<usize, IkfsError> {
          match &self.kind {
              VnodeKind::File { data, .. } => {
                  if offset > data.len() { return Err(IkfsError::InvalidOffset); }
                  let available = &data[offset..];
                  let n = available.len().min(buf.len());
                  buf[..n].copy_from_slice(&available[..n]);
                  Ok(n)
              }
              VnodeKind::Directory => Err(IkfsError::IsDirectory),
              VnodeKind::Pipe { .. } => Err(IkfsError::BrokenPipe),
          }
      }

      /// Write into pipe from sk_a to sk_b.
      pub fn pipe_send_a(&mut self, data: &[u8]) -> Result<usize, IkfsError> {
          match &mut self.kind {
              VnodeKind::Pipe { a_to_b, max_buf, .. } => {
                  if a_to_b.len() + data.len() > *max_buf { return Err(IkfsError::NoSpace); }
                  a_to_b.extend_from_slice(data);
                  self.write_count += 1;
                  Ok(data.len())
              }
              _ => Err(IkfsError::BrokenPipe),
          }
      }

      /// Read from pipe (sk_b receives from sk_a).
      pub fn pipe_recv_b(&mut self, buf: &mut [u8]) -> Result<usize, IkfsError> {
          match &mut self.kind {
              VnodeKind::Pipe { a_to_b, .. } => {
                  let n = a_to_b.len().min(buf.len());
                  buf[..n].copy_from_slice(&a_to_b[..n]);
                  a_to_b.drain(..n);
                  self.read_count += 1;
                  Ok(n)
              }
              _ => Err(IkfsError::BrokenPipe),
          }
      }

      pub fn size(&self) -> usize {
          match &self.kind {
              VnodeKind::File { data, .. } => data.len(),
              VnodeKind::Pipe { a_to_b, b_to_a, .. } => a_to_b.len() + b_to_a.len(),
              VnodeKind::Directory => 0,
          }
      }
  }
  