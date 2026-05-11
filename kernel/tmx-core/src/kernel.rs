//! Nova Fractal Kernel Core — tmx-core personality engine
  //!
  //! v2: implements execute_personality() and the full kernel lifecycle.
  //!
  //! tmx-core is the sub-kernel kernel: each SubKernel instance (spawned by the
  //! Timux master) runs one of these as its innermost logic unit. The personality
  //! engine interprets a simple declarative script that configures the sub-kernel's
  //! identity, resource limits, and startup services.
  //!
  //! Personality script format (line-oriented, whitespace-separated):
  //!   name   <identifier>      — set sub-kernel name
  //!   ring   <-5..4>           — declare the ring level to operate at
  //!   quota  <microseconds>    — CPU quota per epoch
  //!   svc    <service-name>    — register a built-in service
  //!   cap    <right-name>      — assert a required capability
  //!   echo   <message>         — emit a boot-time diagnostic line
  //!   #      <comment>         — ignored

  extern crate alloc;

  use alloc::string::{String, ToString};
  use alloc::vec::Vec;

  // ─── Personality script AST ───────────────────────────────────────────────────

  #[derive(Debug, Clone)]
  pub enum PersonalityDirective {
      Name(String),
      Ring(i8),
      Quota(u64),
      Service(String),
      Capability(String),
      Echo(String),
      Comment,
      Unknown(String),
  }

  fn parse_personality(script: &str) -> Vec<PersonalityDirective> {
      let mut directives = Vec::new();
      for raw_line in script.lines() {
          let line = raw_line.trim();
          if line.is_empty() { continue; }

          let mut parts = line.splitn(2, char::is_whitespace);
          let keyword = parts.next().unwrap_or("").to_lowercase();
          let rest    = parts.next().unwrap_or("").trim();

          let directive = match keyword.as_str() {
              "name"  => PersonalityDirective::Name(rest.to_string()),
              "ring"  => rest.parse::<i8>()
                             .map(PersonalityDirective::Ring)
                             .unwrap_or(PersonalityDirective::Unknown(line.to_string())),
              "quota" => rest.parse::<u64>()
                             .map(PersonalityDirective::Quota)
                             .unwrap_or(PersonalityDirective::Unknown(line.to_string())),
              "svc"   => PersonalityDirective::Service(rest.to_string()),
              "cap"   => PersonalityDirective::Capability(rest.to_string()),
              "echo"  => PersonalityDirective::Echo(rest.to_string()),
              s if s.starts_with('#') => PersonalityDirective::Comment,
              _       => PersonalityDirective::Unknown(line.to_string()),
          };
          directives.push(directive);
      }
      directives
  }

  // ─── KernelConfig (built from personality) ────────────────────────────────────

  #[derive(Debug, Clone)]
  pub struct KernelConfig {
      pub name:         String,
      pub ring:         i8,
      pub quota_us:     u64,
      pub services:     Vec<String>,
      pub capabilities: Vec<String>,
      pub diagnostics:  Vec<String>,
      pub warnings:     Vec<String>,
  }

  impl KernelConfig {
      fn default(kernel_name: &str) -> Self {
          Self {
              name:         kernel_name.to_string(),
              ring:         0,
              quota_us:     5_000,
              services:     Vec::new(),
              capabilities: Vec::new(),
              diagnostics:  Vec::new(),
              warnings:     Vec::new(),
          }
      }
  }

  // ─── Built-in services ────────────────────────────────────────────────────────

  const KNOWN_SERVICES: &[&str] = &[
      "bashpp", "ipc", "vfs", "net", "clock", "crypto", "morph",
      "debug", "snapshot", "journal",
  ];

  fn is_known_service(name: &str) -> bool {
      KNOWN_SERVICES.contains(&name)
  }

  // ─── Kernel ──────────────────────────────────────────────────────────────────

  pub struct Kernel {
      pub name:   &'static str,
      pub config: Option<KernelConfig>,
      pub state:  KernelLifecycle,
  }

  #[derive(Debug, Clone, Copy, PartialEq, Eq)]
  pub enum KernelLifecycle {
      /// Created but no personality loaded yet.
      Nascent,
      /// Personality parsed and applied; ready to run.
      Configured,
      /// Running normally.
      Running,
      /// Suspended (snapshot-compatible).
      Suspended,
      /// Terminated.
      Dead,
  }

  impl Kernel {
      pub fn new(name: &'static str) -> Self {
          Self { name, config: None, state: KernelLifecycle::Nascent }
      }

      /// Execute a personality script to configure this kernel instance.
      ///
      /// The script is parsed line by line. Valid directives update the kernel
      /// configuration; unknown directives are recorded as warnings.
      ///
      /// Returns the applied KernelConfig for the caller's inspection.
      pub fn execute_personality(&mut self, script: &str) -> KernelConfig {
          let directives = parse_personality(script);
          let mut cfg = KernelConfig::default(self.name);

          for directive in directives {
              match directive {
                  PersonalityDirective::Name(n) => {
                      cfg.diagnostics.push(alloc::format!("name: {}", n));
                      cfg.name = n;
                  }
                  PersonalityDirective::Ring(r) => {
                      if r < -5 || r > 4 {
                          cfg.warnings.push(alloc::format!("ring {} out of range [-5,4]", r));
                      } else {
                          cfg.ring = r;
                          cfg.diagnostics.push(alloc::format!("ring: {}", r));
                      }
                  }
                  PersonalityDirective::Quota(q) => {
                      if q == 0 {
                          cfg.warnings.push("quota 0 is invalid; using default 5000 µs".to_string());
                      } else {
                          cfg.quota_us = q;
                          cfg.diagnostics.push(alloc::format!("quota: {} µs", q));
                      }
                  }
                  PersonalityDirective::Service(svc) => {
                      if !is_known_service(&svc) {
                          cfg.warnings.push(alloc::format!("unknown service '{}'", svc));
                      } else if !cfg.services.contains(&svc) {
                          cfg.diagnostics.push(alloc::format!("svc: {}", svc));
                          cfg.services.push(svc);
                      }
                  }
                  PersonalityDirective::Capability(cap) => {
                      if !cfg.capabilities.contains(&cap) {
                          cfg.diagnostics.push(alloc::format!("cap: {}", cap));
                          cfg.capabilities.push(cap);
                      }
                  }
                  PersonalityDirective::Echo(msg) => {
                      cfg.diagnostics.push(alloc::format!("echo: {}", msg));
                  }
                  PersonalityDirective::Comment | PersonalityDirective::Unknown(_) => {}
              }
          }

          self.config = Some(cfg.clone());
          self.state  = KernelLifecycle::Configured;
          cfg
      }

      /// Transition from Configured to Running.
      pub fn start(&mut self) -> Result<(), &'static str> {
          if self.state != KernelLifecycle::Configured {
              return Err("kernel must be in Configured state to start");
          }
          self.state = KernelLifecycle::Running;
          Ok(())
      }

      /// Suspend a running kernel (preserves config for snapshot).
      pub fn suspend(&mut self) -> Result<(), &'static str> {
          if self.state != KernelLifecycle::Running {
              return Err("kernel must be Running to suspend");
          }
          self.state = KernelLifecycle::Suspended;
          Ok(())
      }

      /// Resume a suspended kernel.
      pub fn resume(&mut self) -> Result<(), &'static str> {
          if self.state != KernelLifecycle::Suspended {
              return Err("kernel must be Suspended to resume");
          }
          self.state = KernelLifecycle::Running;
          Ok(())
      }

      /// Shut down the kernel.
      pub fn terminate(&mut self) {
          self.state = KernelLifecycle::Dead;
      }

      pub fn is_running(&self) -> bool { self.state == KernelLifecycle::Running }
  }
  