use crate::models::Codex;
use ratatui::widgets::ListState;

pub enum Screen {
    SpellbookList,
    SpellList,
}

pub struct App {
    pub codex: Codex,
    pub screen: Screen,
    pub spellbook_list_state: ListState,
    pub spell_list_state: ListState,
    pub selected_spellbook: Option<usize>,
}

impl App {
    pub fn new(codex: Codex) -> Self {
        let mut spellbook_list_state = ListState::default();
        spellbook_list_state.select(Some(0));
        Self {
            codex,
            screen: Screen::SpellbookList,
            spellbook_list_state,
            spell_list_state: ListState::default(),
            selected_spellbook: None,
        }
    }

    pub fn next(&mut self) {
        let len = match self.screen {
            Screen::SpellbookList => self.codex.spellbooks.len(),
            Screen::SpellList => self
                .selected_spellbook
                .map(|i| self.codex.spellbooks[i].spell_ids.len())
                .unwrap_or(0),
        };
        let state = match self.screen {
            Screen::SpellbookList => &mut self.spellbook_list_state,
            Screen::SpellList => &mut self.spell_list_state,
        };
        let i = state.selected().map(|i| (i + 1) % len).unwrap_or(0);
        state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let len = match self.screen {
            Screen::SpellbookList => self.codex.spellbooks.len(),
            Screen::SpellList => self
                .selected_spellbook
                .map(|i| self.codex.spellbooks[i].spell_ids.len())
                .unwrap_or(0),
        };
        let state = match self.screen {
            Screen::SpellbookList => &mut self.spellbook_list_state,
            Screen::SpellList => &mut self.spell_list_state,
        };
        let i = state
            .selected()
            .map(|i| if i == 0 { len - 1 } else { i - 1 })
            .unwrap_or(0);
        state.select(Some(i));
    }

    pub fn enter(&mut self) {
        if let Screen::SpellbookList = self.screen {
            if let Some(i) = self.spellbook_list_state.selected() {
                self.selected_spellbook = Some(i);
                self.spell_list_state.select(Some(0));
                self.screen = Screen::SpellList;
            }
        }
    }

    pub fn back(&mut self) {
        if let Screen::SpellList = self.screen {
            self.screen = Screen::SpellbookList;
        }
    }
}

pub fn run(terminal: &mut ratatui::DefaultTerminal, codex: Codex) -> std::io::Result<()> {
    let mut app = App::new(codex);
    loop {
        terminal.draw(|frame| crate::ui::render::render(frame, &mut app))?;

        if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
            if key.kind == crossterm::event::KeyEventKind::Press {
                match key.code {
                    crossterm::event::KeyCode::Char('q') => break,
                    crossterm::event::KeyCode::Esc => app.back(),
                    crossterm::event::KeyCode::Down | crossterm::event::KeyCode::Char('j') => {
                        app.next()
                    }
                    crossterm::event::KeyCode::Up | crossterm::event::KeyCode::Char('k') => {
                        app.previous()
                    }
                    crossterm::event::KeyCode::Enter => app.enter(),
                    _ => {}
                }
            }
        }
    }
    Ok(())
}
