#[derive(Debug, Clone)]
pub struct SpellbookBrowserState {
    pub index: Option<usize>,
    pub scroll: usize,
    pub spines_per_row: usize,
    pub last_width: u16,
    pub last_height: u16,
    pub items_per_row: usize,
}

impl Default for SpellbookBrowserState {
    fn default() -> Self {
        Self {
            index: Some(0),
            scroll: 0,
            spines_per_row: 1,
            last_width: 0,
            last_height: 0,
            items_per_row: 1,
        }
    }
}

impl SpellbookBrowserState {
    pub fn reset(&mut self) {
        self.scroll = 0;
        self.spines_per_row = 1;
        self.last_width = 0;
        self.last_height = 0;
    }
}
