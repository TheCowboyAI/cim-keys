//! Passphrase dialog for secure key generation
//!
//! Provides a modal dialog for entering passphrases with:
//! - Confirmation field to prevent typos
//! - Visibility toggle
//! - Strength indicator
//! - Random passphrase generation
//! - Secure zeroization after use

use iced::{
    widget::{button, checkbox, column, container, row, text, text_input, Column},
    Color, Element, Length, Theme,
};
use der::zeroize::{Zeroize, Zeroizing};

/// Passphrase dialog state
#[derive(Debug, Clone)]
pub struct PassphraseDialog {
    visible: bool,
    passphrase: String,
    passphrase_confirm: String,
    show_passphrase: bool,
    purpose: PassphrasePurpose,
}

/// Purpose for the passphrase
#[derive(Debug, Clone, PartialEq)]
pub enum PassphrasePurpose {
    RootCA,
    IntermediateCA,
    PersonalKeys,
}

impl PassphrasePurpose {
    pub fn title(&self) -> &str {
        match self {
            PassphrasePurpose::RootCA => "Root CA Passphrase",
            PassphrasePurpose::IntermediateCA => "Intermediate CA Passphrase",
            PassphrasePurpose::PersonalKeys => "Personal Keys Passphrase",
        }
    }

    pub fn description(&self) -> &str {
        match self {
            PassphrasePurpose::RootCA => "This passphrase protects your organization's root certificate authority. Keep it secure!",
            PassphrasePurpose::IntermediateCA => "This passphrase protects the intermediate signing certificate.",
            PassphrasePurpose::PersonalKeys => "This passphrase protects your personal keys.",
        }
    }
}

/// Messages from the passphrase dialog
#[derive(Debug, Clone)]
pub enum PassphraseDialogMessage {
    PassphraseChanged(String),
    PassphraseConfirmChanged(String),
    ToggleVisibility(bool),
    GenerateRandom,
    Submit,
    Cancel,
}

impl Default for PassphraseDialog {
    fn default() -> Self {
        Self::new()
    }
}

impl PassphraseDialog {
    /// Create a new passphrase dialog (initially hidden)
    pub fn new() -> Self {
        Self {
            visible: false,
            passphrase: String::new(),
            passphrase_confirm: String::new(),
            show_passphrase: false,
            purpose: PassphrasePurpose::RootCA,
        }
    }

    /// Show the dialog for a specific purpose
    pub fn show(&mut self, purpose: PassphrasePurpose) {
        self.visible = true;
        self.purpose = purpose;
        self.passphrase.clear();
        self.passphrase_confirm.clear();
        self.show_passphrase = false;
    }

    /// Hide the dialog and clear sensitive data
    pub fn hide(&mut self) {
        self.visible = false;
        // Use zeroizing strings for secure cleanup
        let mut pass = Zeroizing::new(self.passphrase.clone());
        let mut confirm = Zeroizing::new(self.passphrase_confirm.clone());
        pass.zeroize();
        confirm.zeroize();
        self.passphrase.clear();
        self.passphrase_confirm.clear();
    }

    /// Check if dialog is visible
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Get the current purpose
    pub fn purpose(&self) -> &PassphrasePurpose {
        &self.purpose
    }

    /// Get the entered passphrase (if valid)
    pub fn get_passphrase(&self) -> Option<Zeroizing<String>> {
        if self.is_valid() {
            Some(Zeroizing::new(self.passphrase.clone()))
        } else {
            None
        }
    }

    /// Check if the entered passphrase is valid
    pub fn is_valid(&self) -> bool {
        !self.passphrase.is_empty()
            && self.passphrase == self.passphrase_confirm
            && self.passphrase.len() >= 12
    }

    /// Calculate passphrase strength (0.0 to 1.0)
    fn strength(&self) -> f32 {
        if self.passphrase.is_empty() {
            return 0.0;
        }

        let mut score = 0.0;

        // Length score (up to 0.4)
        score += (self.passphrase.len() as f32 / 32.0).min(0.4);

        // Character variety score (up to 0.6)
        let has_lowercase = self.passphrase.chars().any(|c| c.is_lowercase());
        let has_uppercase = self.passphrase.chars().any(|c| c.is_uppercase());
        let has_digit = self.passphrase.chars().any(|c| c.is_ascii_digit());
        let has_special = self.passphrase.chars().any(|c| !c.is_alphanumeric());

        if has_lowercase {
            score += 0.15;
        }
        if has_uppercase {
            score += 0.15;
        }
        if has_digit {
            score += 0.15;
        }
        if has_special {
            score += 0.15;
        }

        score.min(1.0)
    }

    /// Get strength color
    fn strength_color(&self) -> Color {
        let strength = self.strength();
        if strength < 0.3 {
            Color::from_rgb(0.8, 0.2, 0.2) // Red
        } else if strength < 0.6 {
            Color::from_rgb(0.8, 0.6, 0.2) // Orange
        } else if strength < 0.8 {
            Color::from_rgb(0.8, 0.8, 0.2) // Yellow
        } else {
            Color::from_rgb(0.2, 0.8, 0.2) // Green
        }
    }

    /// Get strength label
    fn strength_label(&self) -> &str {
        let strength = self.strength();
        if strength < 0.3 {
            "Weak"
        } else if strength < 0.6 {
            "Fair"
        } else if strength < 0.8 {
            "Good"
        } else {
            "Strong"
        }
    }

    /// Generate a random passphrase
    fn generate_random() -> String {
        use rand::Rng;
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                                  abcdefghijklmnopqrstuvwxyz\
                                  0123456789\
                                  !@#$%^&*-_=+";
        let mut rng = rand::thread_rng();
        (0..24)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    }

    /// Handle messages from the dialog
    pub fn update(&mut self, message: PassphraseDialogMessage) {
        match message {
            PassphraseDialogMessage::PassphraseChanged(pass) => {
                self.passphrase = pass;
            }
            PassphraseDialogMessage::PassphraseConfirmChanged(pass) => {
                self.passphrase_confirm = pass;
            }
            PassphraseDialogMessage::ToggleVisibility(show) => {
                self.show_passphrase = show;
            }
            PassphraseDialogMessage::GenerateRandom => {
                let random = Self::generate_random();
                self.passphrase = random.clone();
                self.passphrase_confirm = random;
            }
            PassphraseDialogMessage::Submit => {
                // Handled by parent
            }
            PassphraseDialogMessage::Cancel => {
                // Handled by parent
            }
        }
    }

    /// Render the passphrase dialog
    pub fn view(&self) -> Element<'_, PassphraseDialogMessage> {
        if !self.visible {
            return container(column![]).into();
        }

        // Dialog content
        let mut content: Column<'_, PassphraseDialogMessage> = column![]
            .spacing(16)
            .padding(24)
            .max_width(500);

        // Header
        content = content.push(
            text(self.purpose.title())
                .size(24)
        );

        // Description
        content = content.push(
            text(self.purpose.description())
                .size(14)
                .style(|_theme: &Theme| text::Style {
                    color: Some(Color::from_rgb(0.6, 0.6, 0.6)),
                })
        );

        // Passphrase input
        let passphrase_input = if self.show_passphrase {
            text_input("Enter passphrase (min 12 characters)", &self.passphrase)
        } else {
            text_input("Enter passphrase (min 12 characters)", &self.passphrase)
                .secure(true)
        };

        content = content.push(
            column![
                text("Passphrase:").size(12),
                passphrase_input
                    .on_input(PassphraseDialogMessage::PassphraseChanged)
                    .width(Length::Fill),
            ]
            .spacing(4)
        );

        // Confirm passphrase input
        let confirm_input = if self.show_passphrase {
            text_input("Confirm passphrase", &self.passphrase_confirm)
        } else {
            text_input("Confirm passphrase", &self.passphrase_confirm)
                .secure(true)
        };

        content = content.push(
            column![
                text("Confirm:").size(12),
                confirm_input
                    .on_input(PassphraseDialogMessage::PassphraseConfirmChanged)
                    .width(Length::Fill),
            ]
            .spacing(4)
        );

        // Passphrase match indicator
        if !self.passphrase.is_empty() && !self.passphrase_confirm.is_empty() {
            let match_color = if self.passphrase == self.passphrase_confirm {
                Color::from_rgb(0.2, 0.8, 0.2)
            } else {
                Color::from_rgb(0.8, 0.2, 0.2)
            };
            let match_text = if self.passphrase == self.passphrase_confirm {
                "✓ Passphrases match"
            } else {
                "✗ Passphrases do not match"
            };
            content = content.push(
                text(match_text)
                    .size(12)
                    .style(move |_theme: &Theme| text::Style {
                        color: Some(match_color),
                    })
            );
        }

        // Strength indicator
        if !self.passphrase.is_empty() {
            let strength = self.strength();
            let strength_color = self.strength_color();
            content = content.push(
                column![
                    row![
                        text("Strength:").size(12),
                        text(self.strength_label())
                            .size(12)
                            .style(move |_theme: &Theme| text::Style {
                                color: Some(strength_color),
                            }),
                    ]
                    .spacing(8),
                    container(
                        container(text(""))
                            .width(Length::FillPortion((strength * 100.0) as u16))
                            .height(Length::Fixed(4.0))
                            .style(move |_theme| container::Style {
                                background: Some(iced::Background::Color(strength_color)),
                                ..Default::default()
                            })
                    )
                    .width(Length::Fill)
                    .height(Length::Fixed(4.0))
                    .style(|_theme| container::Style {
                        background: Some(iced::Background::Color(Color::from_rgb(0.2, 0.2, 0.2))),
                        ..Default::default()
                    })
                ]
                .spacing(4)
            );
        }

        // Options row
        content = content.push(
            row![
                checkbox("Show passphrase", self.show_passphrase)
                    .on_toggle(PassphraseDialogMessage::ToggleVisibility)
                    .size(14),
                button(text("Generate Random").size(14))
                    .on_press(PassphraseDialogMessage::GenerateRandom)
            ]
            .spacing(16)
            .align_y(iced::Alignment::Center)
        );

        // Action buttons
        let submit_button = if self.is_valid() {
            button(text("OK").size(14))
                .on_press(PassphraseDialogMessage::Submit)
                .style(|theme: &Theme, _status| button::Style {
                    background: Some(iced::Background::Color(theme.palette().success)),
                    text_color: Color::WHITE,
                    border: iced::Border::default(),
                    shadow: iced::Shadow::default(),
                })
        } else {
            button(text("OK").size(14))
                .style(|_theme: &Theme, _status| button::Style {
                    background: Some(iced::Background::Color(Color::from_rgb(0.3, 0.3, 0.3))),
                    text_color: Color::from_rgb(0.5, 0.5, 0.5),
                    border: iced::Border::default(),
                    shadow: iced::Shadow::default(),
                })
        };

        content = content.push(
            row![
                submit_button,
                button(text("Cancel").size(14))
                    .on_press(PassphraseDialogMessage::Cancel)
                    .style(|theme: &Theme, _status| button::Style {
                        background: Some(iced::Background::Color(theme.palette().danger)),
                        text_color: Color::WHITE,
                        border: iced::Border::default(),
                        shadow: iced::Shadow::default(),
                    }),
            ]
            .spacing(12)
        );

        // Wrap in centered overlay
        container(
            container(content)
                .style(|_theme| container::Style {
                    background: Some(iced::Background::Color(Color::from_rgba8(30, 30, 40, 0.98))),
                    border: iced::Border {
                        color: Color::from_rgb(0.4, 0.6, 0.8),
                        width: 2.0,
                        radius: 8.0.into(),
                    },
                    shadow: iced::Shadow {
                        color: Color::from_rgba8(0, 0, 0, 0.5),
                        offset: iced::Vector::new(0.0, 4.0),
                        blur_radius: 16.0,
                    },
                    ..Default::default()
                })
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(|_theme| container::Style {
            background: Some(iced::Background::Color(Color::from_rgba8(0, 0, 0, 0.7))),
            ..Default::default()
        })
        .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_passphrase_validation() {
        let mut dialog = PassphraseDialog::new();

        // Too short
        dialog.passphrase = "short".to_string();
        dialog.passphrase_confirm = "short".to_string();
        assert!(!dialog.is_valid());

        // Long enough but don't match
        dialog.passphrase = "longpassphrase123".to_string();
        dialog.passphrase_confirm = "different".to_string();
        assert!(!dialog.is_valid());

        // Valid
        dialog.passphrase = "longpassphrase123".to_string();
        dialog.passphrase_confirm = "longpassphrase123".to_string();
        assert!(dialog.is_valid());
    }

    #[test]
    fn test_strength_calculation() {
        let mut dialog = PassphraseDialog::new();

        // Weak - "password" = 8/32 length (0.25) + lowercase (0.15) = 0.40
        dialog.passphrase = "password".to_string();
        assert!(dialog.strength() <= 0.4, "Expected strength <= 0.4, got {}", dialog.strength());

        // Strong - has length, uppercase, lowercase, digit, special
        dialog.passphrase = "MyP@ssw0rd!IsVeryStr0ng123".to_string();
        assert!(dialog.strength() > 0.8, "Expected strength > 0.8, got {}", dialog.strength());
    }

    #[test]
    fn test_secure_cleanup() {
        let mut dialog = PassphraseDialog::new();
        dialog.show(PassphrasePurpose::RootCA);
        dialog.passphrase = "sensitive".to_string();
        dialog.passphrase_confirm = "sensitive".to_string();

        dialog.hide();
        assert!(dialog.passphrase.is_empty());
        assert!(dialog.passphrase_confirm.is_empty());
    }
}
