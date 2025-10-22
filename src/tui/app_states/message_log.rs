use ratatui::style::{Color, Style};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MessageType {
    Info,
    Warning,
    Error,
}

impl MessageType {
    /// Returns the style for this message type.
    pub fn style(&self) -> Style {
        match self {
            MessageType::Info => Style::default(),
            MessageType::Warning => Style::default().fg(Color::Yellow),
            MessageType::Error => Style::default().fg(Color::Red),
        }
    }
}

#[derive(Clone, Debug)]
pub struct AppMessage {
    pub text: String,
    pub msg_type: MessageType,
}

#[derive(Clone, Debug)]
pub struct OutputLog {
    pub messages: Vec<AppMessage>,
    pub scroll_position: usize,
    window_height: usize,
}

impl OutputLog {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            scroll_position: 0,
            window_height: 1, // Default, will be updated by UI
        }
    }

    /// Clears all messages from the log.
    pub fn clear(&mut self) {
        self.messages.clear();
        self.scroll_position = 0;
    }

    /// Sets the visible height of the window, clamping the scroll position.
    pub fn set_window_height(&mut self, height: usize) {
        // Subtract 2 for borders
        self.window_height = height.saturating_sub(2).max(1);
        self.clamp_scroll();
    }

    /// Clamps the scroll position to valid bounds.
    fn clamp_scroll(&mut self) {
        let max_scroll = self
            .messages
            .len()
            .saturating_sub(self.window_height);
        self.scroll_position = self.scroll_position.min(max_scroll);
    }

    /// Scrolls up by a number of lines.
    pub fn scroll_up(&mut self, lines: usize) {
        self.scroll_position = self.scroll_position.saturating_sub(lines);
    }

    /// Scrolls down by a number of lines.
    pub fn scroll_down(&mut self, lines: usize) {
        self.scroll_position = self.scroll_position.saturating_add(lines);
        self.clamp_scroll();
    }

    /// Scrolls to the bottom of the log.
    pub fn scroll_to_bottom(&mut self) {
        self.scroll_position = self
            .messages
            .len()
            .saturating_sub(self.window_height);
    }

    /// Adds a new info message.
    pub fn info(&mut self, message: String) {
        self.messages.push(AppMessage {
            text: message,
            msg_type: MessageType::Info,
        });
        self.scroll_to_bottom(); // Auto-scroll
    }

    /// Adds a new warning message.
    pub fn warn(&mut self, message: String) {
        self.messages.push(AppMessage {
            text: message,
            msg_type: MessageType::Warning,
        });
        self.scroll_to_bottom(); // Auto-scroll
    }

    /// Adds a new error message.
    pub fn error(&mut self, message: String) {
        self.messages.push(AppMessage {
            text: message,
            msg_type: MessageType::Error,
        });
        self.scroll_to_bottom(); // Auto-scroll
    }
}

impl Default for OutputLog {
    fn default() -> Self {
        Self::new()
    }
}