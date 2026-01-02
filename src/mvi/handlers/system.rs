// Copyright (c) 2025 - Cowboy AI, LLC.

//! System Intent Handlers
//!
//! Pure handlers for system-level intents (file picker, clipboard, etc.).
//!
//! ## Subject Patterns
//!
//! - `system.file.selected` - File selected from picker
//! - `system.file.cancelled` - File picker cancelled
//! - `system.clipboard.*` - Clipboard operations
//! - `system.error` - System-level errors

use super::{Intent, Model, HandlerResult};
use iced::Task;
use std::path::PathBuf;

/// Handle file selected from system picker
pub fn handle_file_selected(model: Model, path: PathBuf) -> HandlerResult {
    (model, Task::perform(async move { Intent::UiLoadDomainClicked { path } }, |i| i))
}

/// Handle file picker cancelled
pub fn handle_file_picker_cancelled(model: Model) -> HandlerResult {
    let updated = model.with_status_message("File selection cancelled".to_string());
    (updated, Task::none())
}

/// Handle system error occurred
pub fn handle_system_error(model: Model, context: String, error: String) -> HandlerResult {
    let updated = model
        .with_error(Some(format!("{}: {}", context, error)));

    (updated, Task::none())
}

/// Handle clipboard updated
pub fn handle_clipboard_updated(model: Model, text: String) -> HandlerResult {
    let updated = model.with_status_message(format!(
        "Clipboard updated ({} chars)",
        text.len()
    ));
    (updated, Task::none())
}
