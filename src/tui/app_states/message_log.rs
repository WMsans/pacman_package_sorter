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
}

impl OutputLog {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
        }
    }

    /// Adds a new info message.
    pub fn info(&mut self, message: String) {
        self.messages.push(AppMessage {
            text: message,
            msg_type: MessageType::Info,
        });
    }

    /// Adds a new warning message.
    pub fn warn(&mut self, message: String) {
        self.messages.push(AppMessage {
            text: message,
            msg_type: MessageType::Warning,
        });
    }

    /// Adds a new error message.
    pub fn error(&mut self, message: String) {
        self.messages.push(AppMessage {
            text: message,
            msg_type: MessageType::Error,
        });
    }
}

impl Default for OutputLog {
    fn default() -> Self {
        Self::new()
    }
}