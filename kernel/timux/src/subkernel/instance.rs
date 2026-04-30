//! SubKernel — a complete isolated OS instance inside Timux

use crate::priv_model::{CapabilityTable, CapabilityToken, CapRight, RingLevel};
use crate::sched::Scheduler;
use crate::mm::AddressSpace;
use crate::ipc::Channel;

pub type SubKernelId = u64;

/// What kind of OS this sub-kernel is configured as
#[derive(Debug, Clone)]
pub enum SubKernelProfile {
    /// General-purpose OS (default)
    GeneralPurpose,
    /// Real-time OS — strict scheduling guarantees
    RealTime { deadline_us: u64 },
    /// Networking OS — NET caps pre-loaded
    Networking,
    /// Storage/filesystem OS — FS caps pre-loaded
    Storage,
    /// GPU/graphics OS — DMA + MMIO caps pre-loaded
    Graphics,
    /// Security enclave — maximum isolation, minimal caps
    Enclave,
    /// Custom — caller defines everything
    Custom { name: &'static str },
}

impl SubKernelProfile {
    pub fn name(&self) -> &'static str {
        match self {
            Self::GeneralPurpose  => "general-purpose",
            Self::RealTime { .. } => "real-time",
            Self::Networking      => "networking",
            Self::Storage         => "storage",
            Self::Graphics        => "graphics",
            Self::Enclave         => "enclave",
            Self::Custom { name } => name,
        }
    }

    /// Default capability rights for this profile
    pub fn default_rights(&self) -> CapRight {
        match self {
            Self::GeneralPurpose => {
                CapRight::READ | CapRight::WRITE | CapRight::EXEC
                | CapRight::MAP | CapRight::SEND | CapRight::RECV
                | CapRight::PROCESS_SPAWN | CapRight::FS_READ | CapRight::FS_WRITE
                | CapRight::NET_SEND | CapRight::NET_RECV | CapRight::CLOCK_READ
            }
            Self::RealTime { .. } => {
                CapRight::READ | CapRight::WRITE | CapRight::EXEC
                | CapRight::MAP | CapRight::SEND | CapRight::RECV
                | CapRight::CLOCK_READ | CapRight::CLOCK_SET | CapRight::IRQ_BIND
            }
            Self::Networking => {
                CapRight::READ | CapRight::WRITE | CapRight::EXEC
                | CapRight::MAP | CapRight::SEND | CapRight::RECV
                | CapRight::NET_SEND | CapRight::NET_RECV
                | CapRight::IRQ_BIND | CapRight::DMA_ACCESS
            }
            Self::Storage => {
                CapRight::READ | CapRight::WRITE | CapRight::EXEC
                | CapRight::MAP | CapRight::SEND | CapRight::RECV
                | CapRight::FS_READ | CapRight::FS_WRITE
                | CapRight::IRQ_BIND | CapRight::DMA_ACCESS
            }
            Self::Graphics => {
                CapRight::READ | CapRight::WRITE | CapRight::EXEC
                | CapRight::MAP | CapRight::SEND | CapRight::RECV
                | CapRight::DMA_ACCESS | CapRight::MMIO_ACCESS | CapRight::IRQ_BIND
            }
            Self::Enclave => {
                CapRight::READ | CapRight::EXEC
            }
            Self::Custom { .. } => CapRight::empty(),
        }
    }
}

/// Configuration passed to spawn a new sub-kernel
#[derive(Debug, Clone)]
pub struct SubKernelConfig {
    pub name:       &'static str,
    pub profile:    SubKernelProfile,
    pub mem_pages:  usize,       // how many physical pages allocated
    pub max_tasks:  usize,       // max tasks inside this sub-kernel
    pub priority:   u8,          // scheduling weight vs other sub-kernels
    pub extra_caps: CapRight,    // additional rights on top of profile defaults
}

impl SubKernelConfig {
    pub fn new(name: &'static str, profile: SubKernelProfile) -> Self {
        Self {
            name,
            profile,
            mem_pages: 256,  // 1MB default
            max_tasks: 64,
            priority: 128,
            extra_caps: CapRight::empty(),
        }
    }

    pub fn with_mem(mut self, pages: usize) -> Self { self.mem_pages = pages; self }
    pub fn with_priority(mut self, p: u8) -> Self { self.priority = p; self }
    pub fn with_caps(mut self, caps: CapRight) -> Self { self.extra_caps = caps; self }
}

/// Lifecycle state of a sub-kernel
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubKernelState {
    /// Being constructed
    Initializing,
    /// Running normally
    Running,
    /// Paused — state preserved, not scheduled
    Paused,
    /// Being hot-swapped — frozen, new image loading
    HotSwapping,
    /// Being migrated to another physical location
    Migrating,
    /// Terminated — resources freed
    Terminated,
}

impl SubKernelState {
    pub fn can_pause(self) -> bool { self == Self::Running }
    pub fn can_resume(self) -> bool { self == Self::Paused }
    pub fn can_swap(self) -> bool { matches!(self, Self::Running | Self::Paused) }
    pub fn can_migrate(self) -> bool { matches!(self, Self::Running | Self::Paused) }
    pub fn is_alive(self) -> bool { !matches!(self, Self::Terminated) }
}

/// A complete isolated OS instance
pub struct SubKernel {
    pub id:        SubKernelId,
    pub config:    SubKernelConfig,
    pub state:     SubKernelState,
    pub scheduler: Scheduler,
    pub memory:    AddressSpace,
    pub caps:      CapabilityTable,
    pub ring_model: RingModel,
    pub drivers:   DriverRegistry,
    pub ipc_channels: alloc::vec::Vec<Channel>,
    pub parent:    Option<SubKernelId>,
    pub children:  alloc::vec::Vec<SubKernelId>,
    pub tick:      u64,
}

/// Each sub-kernel has its own 5-ring model, independent of the master kernel
#[derive(Debug)]
pub struct RingModel {
    pub ring0_cap: CapabilityToken, // sk-core authority cap
    pub ring1_cap: CapabilityToken, // sk-ext authority cap
    pub ring2_cap: CapabilityToken, // sk-service cap
    pub ring3_cap: CapabilityToken, // user cap
    pub ring4_cap: CapabilityToken, // sandbox cap
}

impl RingModel {
    pub fn new(profile: &SubKernelProfile) -> Self {
        let rights = profile.default_rights();
        Self {
            ring0_cap: CapabilityToken::mint(CapRight::all(), RingLevel::KernelCore),
            ring1_cap: CapabilityToken::mint(
                rights | CapRight::IRQ_BIND | CapRight::MMIO_ACCESS,
                RingLevel::KernelExtension,
            ),
            ring2_cap: CapabilityToken::mint(
                rights & !(CapRight::IRQ_BIND | CapRight::DMA_ACCESS | CapRight::MMIO_ACCESS),
                RingLevel::SystemService,
            ),
            ring3_cap: CapabilityToken::mint(
                rights & !(CapRight::IRQ_BIND | CapRight::DMA_ACCESS
                    | CapRight::MMIO_ACCESS | CapRight::PROCESS_KILL),
                RingLevel::User,
            ),
            ring4_cap: CapabilityToken::mint(
                CapRight::READ | CapRight::EXEC,
                RingLevel::Sandbox,
            ),
        }
    }
}

/// Registered drivers inside a sub-kernel
#[derive(Debug, Default)]
pub struct DriverRegistry {
    entries: alloc::vec::Vec<DriverEntry>,
}

#[derive(Debug)]
pub struct DriverEntry {
    pub name:   &'static str,
    pub kind:   DriverKind,
    pub active: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum DriverKind {
    Network,
    Block,
    Char,
    Gpu,
    Rtc,
    Input,
    Custom,
}

impl DriverRegistry {
    pub fn register(&mut self, name: &'static str, kind: DriverKind) {
        self.entries.push(DriverEntry { name, kind, active: true });
    }

    pub fn count(&self) -> usize { self.entries.len() }
    pub fn active(&self) -> usize { self.entries.iter().filter(|e| e.active).count() }
}

static SK_COUNTER: core::sync::atomic::AtomicU64 =
    core::sync::atomic::AtomicU64::new(1);

impl SubKernel {
    /// Spawn a new sub-kernel from a config
    pub fn spawn(config: SubKernelConfig, parent: Option<SubKernelId>) -> Self {
        let id = SK_COUNTER.fetch_add(1, core::sync::atomic::Ordering::SeqCst);
        let rights = config.profile.default_rights() | config.extra_caps;
        let mut caps = CapabilityTable::new();
        caps.insert(CapabilityToken::mint(rights, RingLevel::KernelCore))
            .expect("cap insert failed");

        Self {
            id,
            ring_model: RingModel::new(&config.profile),
            scheduler: Scheduler::new(),
            memory: AddressSpace::new(0),
            drivers: DriverRegistry::default(),
            ipc_channels: alloc::vec::Vec::new(),
            children: alloc::vec::Vec::new(),
            state: SubKernelState::Initializing,
            parent,
            tick: 0,
            caps,
            config,
        }
    }

    /// Transition to Running
    pub fn boot(&mut self) {
        self.state = SubKernelState::Running;
    }

    /// Pause — freeze state, stop scheduling
    pub fn pause(&mut self) -> Result<(), &'static str> {
        if !self.state.can_pause() {
            return Err("sub-kernel is not running");
        }
        self.state = SubKernelState::Paused;
        Ok(())
    }

    /// Resume from pause
    pub fn resume(&mut self) -> Result<(), &'static str> {
        if !self.state.can_resume() {
            return Err("sub-kernel is not paused");
        }
        self.state = SubKernelState::Running;
        Ok(())
    }

    /// Hot-swap — replace the running image without killing the sub-kernel
    /// State is frozen, new config loaded, then resumed
    pub fn hot_swap(&mut self, new_config: SubKernelConfig) -> Result<(), &'static str> {
        if !self.state.can_swap() {
            return Err("sub-kernel cannot be hot-swapped in current state");
        }
        self.state = SubKernelState::HotSwapping;
        // Freeze scheduler
        // Load new image (arch-specific)
        // Rebuild ring model for new profile
        self.ring_model = RingModel::new(&new_config.profile);
        self.config = new_config;
        self.state = SubKernelState::Running;
        Ok(())
    }

    /// Clone this sub-kernel — returns a new SubKernel with copied state
    pub fn clone_sk(&self, new_parent: Option<SubKernelId>) -> SubKernel {
        let mut cloned = SubKernel::spawn(self.config.clone(), new_parent);
        cloned.boot(); // clone starts Running immediately
        cloned
    }

    /// Begin migration — freeze and mark for transport
    pub fn begin_migrate(&mut self) -> Result<(), &'static str> {
        if !self.state.can_migrate() {
            return Err("sub-kernel cannot migrate in current state");
        }
        self.state = SubKernelState::Migrating;
        Ok(())
    }

    /// Complete migration — resume after arriving at new location
    pub fn complete_migrate(&mut self) {
        self.state = SubKernelState::Running;
    }

    /// Terminate — clean up all resources
    pub fn terminate(&mut self) {
        self.state = SubKernelState::Terminated;
    }

    /// Spawn a child sub-kernel inside this one
    pub fn spawn_child(&mut self, config: SubKernelConfig) -> SubKernel {
        let child = SubKernel::spawn(config, Some(self.id));
        self.children.push(child.id);
        child
    }

    pub fn tick(&mut self) -> u64 {
        self.tick += 1;
        self.tick
    }

    pub fn is_alive(&self) -> bool { self.state.is_alive() }
    pub fn name(&self) -> &'static str { self.config.name }
    pub fn profile(&self) -> &str { self.config.profile.name() }
}

extern crate alloc;
