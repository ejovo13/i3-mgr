pub(crate) type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub(crate) use ratatui::widgets::{Block, List};
pub(crate) use serde::{Deserialize, Serialize};
pub(crate) use std::collections::HashMap;
pub(crate) use std::process::Stdio;
pub(crate) use std::u64;

pub(crate) use ratatui::{
    backend::CrosstermBackend,
    crossterm::{
        event::{self, KeyCode, KeyEventKind},
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
        ExecutableCommand,
    },
    style,
    style::Stylize,
    widgets::Paragraph,
    Terminal,
};

pub(crate) use ratatui::style::{
    palette::tailwind::{BLUE, GREEN, SLATE},
    Color, Modifier, Style,
};
