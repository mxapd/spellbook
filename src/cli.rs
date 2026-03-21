use std::env;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AppMode {
    #[default]
    Browse,
    AddSpell,
}

#[derive(Debug, Clone, Default)]
pub struct CliArgs {
    pub mode: AppMode,
}

impl CliArgs {
    pub fn parse() -> Self {
        let args: Vec<String> = env::args().collect();
        let mode = if args.iter().any(|a| a == "--add") {
            AppMode::AddSpell
        } else {
            AppMode::Browse
        };

        Self { mode }
    }
}
