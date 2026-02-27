use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    Frame,
    style::{Color, Style},
    text::Text,
    widgets::Paragraph,
};

fn main() -> std::io::Result<()> {
    // Initialize the terminal: switches to the alternate screen buffer
    // and enables raw mode (keypresses are sent directly, not line-buffered).
    let mut terminal = ratatui::init();

    let result = app(&mut terminal);

    // Restore the terminal: exits alternate screen and disables raw mode.
    // Always call this, even if app() errored, so the user's terminal isn't broken.
    ratatui::restore();

    result
}

fn app(terminal: &mut ratatui::DefaultTerminal) -> std::io::Result<()> {
    loop {
        // terminal.draw() clears the screen and calls your closure with a Frame.
        // The Frame is how you place widgets onto the screen each tick.
        // Ratatui uses an "immediate mode" model: you redraw everything each frame
        // rather than mutating persistent UI state.
        terminal.draw(|frame| render(frame))?;

        if handle_input()? {
            return Ok(());
        }
    }
}

fn render(frame: &mut Frame) {
    // frame.area() returns a Rect representing the full terminal size.
    // A Rect has x, y, width, height — it defines where a widget is drawn.
    let area = frame.area();

    // Widgets are structs that implement the Widget trait.
    // Paragraph is the most common text widget.
    let greeting =
        Paragraph::new(Text::raw("Hello, world!")).style(Style::default().fg(Color::Green));

    // render_widget() draws a widget into a given Rect.
    // Nothing appears on screen until draw() finishes and flushes the buffer.
    frame.render_widget(greeting, area);
}

fn handle_input() -> std::io::Result<bool> {
    // event::read() blocks until the user presses a key, resizes the terminal, etc.
    // In a real app you might use event::poll() with a timeout for animations.
    if let Event::Key(key) = event::read()? {
        // KeyCode represents the actual key pressed.
        // key.kind can distinguish Press vs Release vs Repeat if needed.
        return Ok(key.code == KeyCode::Char('q'));
    }

    Ok(false)
}

