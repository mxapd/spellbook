mod app;
mod models;
mod persistence;
mod ui;

use ratatui;

fn main() -> std::io::Result<()> {
    // load codex from disk
    println!("started program");
    let codex = persistence::Archivist::load("codex.json").expect("Failed to load codex");
    println!("loaded codex");

    println!("spells loaded: {}", codex.spells.len());
    println!("spellbooks loaded: {}", codex.spellbooks.len());

    // create tui
    let mut terminal = ratatui::init();

    let result = app::run(&mut terminal, codex);
    ratatui::restore();
    result
}
