// Copyright (c) 2025 - Cowboy AI, LLC.

//! Error Intent Handlers
//!
//! Pure handlers for error-related intents.
//!
//! ## Subject Patterns
//!
//! - `error.occurred` - Error occurred in some context
//! - `error.dismissed` - Error dismissed by user

use super::{Model, HandlerResult};
use iced::Task;

/// Handle error occurred
pub fn handle_error_occurred(model: Model, context: String, message: String) -> HandlerResult {
    let updated = model
        .with_error(Some(format!("{}: {}", context, message)));

    (updated, Task::none())
}

/// Handle error dismissed
pub fn handle_error_dismissed(model: Model, error_id: String) -> HandlerResult {
    let updated = model
        .with_error(None)
        .with_status_message(format!("Error dismissed: {}", error_id));
    (updated, Task::none())
}
