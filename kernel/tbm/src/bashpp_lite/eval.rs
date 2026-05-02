//! Bash++ Lite Evaluator for TBM

use crate::bashpp_lite::parser::Command;
use crate::graphics::SovereignDashboard;

pub struct Evaluator<'a, 'b> {
    dashboard: &'a mut SovereignDashboard<'b>,
}

impl<'a, 'b> Evaluator<'a, 'b> {
    pub fn new(dashboard: &'a mut SovereignDashboard<'b>) -> Self {
        Self { dashboard }
    }

    pub fn eval(&mut self, cmds: &[Option<Command>]) {
        for cmd_opt in cmds {
            if let Some(cmd) = cmd_opt {
                match cmd {
                    Command::SetTheme(_theme) => {
                        // Logic to change theme in dashboard
                    }
                    Command::AddEntry { name: _, profile: _ } => {
                        // In TBM context, name and profile
                    }
                    _ => {}
                }
            }
        }
    }
}
