use crate::models::Codex;

pub struct State {
    pub codex: Codex,
}

impl State {
    pub fn new(codex: Codex) -> Self {
        Self { codex }
    }
}
