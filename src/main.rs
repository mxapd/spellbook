mod models;

use models::{Archivist, Codex};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

fn main() -> std::io::Result<()> {
    // load codex from disk
    println!("started program");
    let codex = Archivist::load("codex.json").expect("Failed to load codex");
    println!("loaded codex");

    println!("spells loaded: {}", codex.spells.len());
    println!("spellbooks loaded: {}", codex.spellbooks.len());

    // draw ui:
    let mut terminal = ratatui::init();
    let result = app(&mut terminal, &codex);
    ratatui::restore();
    result
}

fn app(terminal: &mut DefaultTerminal, codex: &Codex) -> std::io::Result<()> {
    loop {
        terminal.draw(|frame| render(frame, codex))?;
        if crossterm::event::read()?.is_key_press() {
            break Ok(());
        }
    }
}

fn render(frame: &mut Frame, codex: &Codex) {
    // split the screen into 3 vertical sections
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // title bar
            Constraint::Min(0),    // list (takes remaining space)
            Constraint::Length(1), // help bar
        ])
        .split(frame.area());

    // title
    let title = Paragraph::new(" * spellbooks * ")
        .style(Style::default().fg(Color::Yellow))
        .alignment(ratatui::layout::HorizontalAlignment::Center);
    frame.render_widget(title, layout[0]);

    // spellbook list
    let items: Vec<ListItem> = codex
        .spellbooks
        .iter()
        .map(|sb| {
            let line = Line::from(sb.name.as_str()).centered();
            ListItem::new(line)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL))
        .highlight_symbol("> ")
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    frame.render_widget(list, layout[1]);

    // help bar
    let help = Paragraph::new(" [q] quit").style(Style::default().fg(Color::DarkGray));
    frame.render_widget(help, layout[2]);
}
