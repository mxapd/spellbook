mod models;

use models::Archivist;

fn main() {
    // load codex from disk
    println!("started program");
    let codex = Archivist::load("codex.json").expect("Failed to load codex");
    println!("loaded codex");
}
