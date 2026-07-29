pub struct ZenMode {
    pub active: bool,
    pub saved_state: SavedUIState,
}

pub struct SavedUIState {
    pub file_explorer: bool,
    pub terminal: bool,
    pub ai_panel: bool,
    pub status_bar: bool,
    pub menu_bar: bool,
    pub line_numbers: bool,
    pub minimap: bool,
}

impl ZenMode {
    pub fn new() -> Self {
        Self {
            active: false,
            saved_state: SavedUIState {
                file_explorer: true,
                terminal: false,
                ai_panel: false,
                status_bar: true,
                menu_bar: true,
                line_numbers: true,
                minimap: true,
            },
        }
    }

    pub fn toggle(&mut self) {
        self.active = !self.active;
    }
}
