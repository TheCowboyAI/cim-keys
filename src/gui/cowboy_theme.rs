//! The Cowboy AI theme for cim-keys GUI
//!
//! This theme matches the styling from thecowboy.ai website
//! with beautiful gradients, glass morphism, and dark aesthetics

use iced::{
    Background, Border, Color, Shadow,
    gradient::{Gradient, Linear},
    widget::{button, container, text_input},
};

/// The Cowboy AI theme colors and gradients
pub struct CowboyTheme;

impl CowboyTheme {
    /// Primary blue gradient (used for buttons and accents)
    pub fn primary_gradient() -> Background {
        Background::Gradient(Gradient::Linear(
            Linear::new(135.0)
                .add_stop(0.0, Color::from_rgb(0.118, 0.235, 0.447))  // #1e3c72
                .add_stop(1.0, Color::from_rgb(0.165, 0.322, 0.596))  // #2a5298
        ))
    }

    /// Hero gradient (blue to purple)
    pub fn hero_gradient() -> Background {
        Background::Gradient(Gradient::Linear(
            Linear::new(45.0)
                .add_stop(0.0, Color::from_rgb(0.4, 0.49, 0.92))
                .add_stop(1.0, Color::from_rgb(0.45, 0.29, 0.64))
        ))
    }

    /// Security gradient (for key operations)
    pub fn security_gradient() -> Background {
        Background::Gradient(Gradient::Linear(
            Linear::new(135.0)
                .add_stop(0.0, Color::from_rgb(0.31, 0.68, 0.99))
                .add_stop(1.0, Color::from_rgb(0.0, 0.95, 0.99))
        ))
    }

    /// Success gradient (for successful operations)
    pub fn success_gradient() -> Background {
        Background::Gradient(Gradient::Linear(
            Linear::new(135.0)
                .add_stop(0.0, Color::from_rgb(0.19, 0.81, 0.82))
                .add_stop(1.0, Color::from_rgb(0.2, 0.03, 0.4))
        ))
    }

    /// Warning gradient (for important operations)
    pub fn warning_gradient() -> Background {
        Background::Gradient(Gradient::Linear(
            Linear::new(135.0)
                .add_stop(0.0, Color::from_rgb(0.98, 0.44, 0.60))
                .add_stop(1.0, Color::from_rgb(0.99, 0.88, 0.25))
        ))
    }

    /// Dark background (main background) - matches www-egui
    pub fn dark_background() -> Background {
        Background::Gradient(Gradient::Linear(
            Linear::new(180.0)
                .add_stop(0.0, Color::from_rgb(0.0078, 0.278, 0.447))  // Teal blue #024772
                .add_stop(1.0, Color::from_rgb(0.0, 0.0, 0.314))       // Navy #000050
        ))
    }

    /// Glass morphism background (for cards) - lighter frosted glass
    pub fn glass_background() -> Background {
        Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.12))
    }

    /// Glass morphism background (darker variant) - enhanced translucency
    pub fn glass_dark_background() -> Background {
        Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.4))
    }

    /// Glass morphism background (medium variant) - for nested cards
    pub fn glass_medium_background() -> Background {
        Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.25))
    }

    /// White radial gradient for logo background - 30% alpha at center to 0% at edges
    pub fn logo_radial_gradient() -> Background {
        Background::Gradient(Gradient::Linear(
            Linear::new(0.0)
                .add_stop(0.0, Color::from_rgba(1.0, 1.0, 1.0, 0.3))   // 30% white at center
                .add_stop(1.0, Color::from_rgba(1.0, 1.0, 1.0, 0.0))   // 0% at edges
        ))
    }

    /// Pastel teal card background (soft, colorful)
    pub fn pastel_teal_background() -> Background {
        Background::Color(Color::from_rgba(0.4, 0.7, 0.75, 0.3))  // Soft teal with transparency
    }

    /// Pastel coral card background (accent color)
    pub fn pastel_coral_background() -> Background {
        Background::Color(Color::from_rgba(0.95, 0.7, 0.6, 0.3))  // Soft coral/peach
    }

    /// Pastel cream card background (neutral accent)
    pub fn pastel_cream_background() -> Background {
        Background::Color(Color::from_rgba(0.95, 0.92, 0.85, 0.3))  // Warm cream
    }

    /// Pastel mint card background (fresh accent)
    pub fn pastel_mint_background() -> Background {
        Background::Color(Color::from_rgba(0.7, 0.9, 0.8, 0.3))  // Soft mint green
    }

    /// Text colors
    pub fn text_primary() -> Color {
        Color::WHITE
    }

    pub fn text_secondary() -> Color {
        Color::from_rgba(1.0, 1.0, 1.0, 0.8)
    }

    pub fn text_muted() -> Color {
        Color::from_rgba(1.0, 1.0, 1.0, 0.5)
    }

    /// Border colors
    pub fn border_color() -> Color {
        Color::from_rgba(1.0, 1.0, 1.0, 0.2)
    }

    pub fn border_hover_color() -> Color {
        Color::from_rgba(1.0, 1.0, 1.0, 0.4)
    }

    /// Create a glowing shadow
    pub fn glow_shadow() -> Shadow {
        Shadow {
            color: Color::from_rgba(0.165, 0.322, 0.596, 0.3),
            offset: iced::Vector::new(0.0, 0.0),
            blur_radius: 30.0,
        }
    }

    /// Button shadow
    pub fn button_shadow() -> Shadow {
        Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.3),
            offset: iced::Vector::new(0.0, 4.0),
            blur_radius: 10.0,
        }
    }
}

/// Custom theme implementation
pub struct CowboyAppTheme;

impl CowboyAppTheme {
    /// Get a custom button style
    pub fn primary_button() -> impl Fn(&iced::Theme, button::Status) -> button::Style {
        |_theme, status| {
            let (background, text_color, border_color, shadow) = match status {
                button::Status::Active => (
                    CowboyTheme::primary_gradient(),
                    CowboyTheme::text_primary(),
                    CowboyTheme::border_color(),
                    CowboyTheme::button_shadow(),
                ),
                button::Status::Hovered => (
                    CowboyTheme::primary_gradient(),
                    CowboyTheme::text_primary(),
                    CowboyTheme::border_hover_color(),
                    Shadow {
                        color: Color::from_rgba(0.165, 0.322, 0.596, 0.4),
                        offset: iced::Vector::new(0.0, 6.0),
                        blur_radius: 20.0,
                    },
                ),
                button::Status::Pressed => (
                    CowboyTheme::primary_gradient(),
                    CowboyTheme::text_primary(),
                    CowboyTheme::border_hover_color(),
                    Shadow {
                        color: Color::from_rgba(0.0, 0.0, 0.0, 0.2),
                        offset: iced::Vector::new(0.0, 2.0),
                        blur_radius: 5.0,
                    },
                ),
                button::Status::Disabled => (
                    Background::Color(Color::from_rgba(0.5, 0.5, 0.5, 0.3)),
                    CowboyTheme::text_muted(),
                    Color::from_rgba(0.5, 0.5, 0.5, 0.2),
                    Shadow::default(),
                ),
            };

            button::Style {
                background: Some(background),
                text_color,
                border: Border {
                    color: border_color,
                    width: 1.0,
                    radius: 10.0.into(),
                },
                shadow,
            }
        }
    }

    /// Glass morphism button style
    pub fn glass_button() -> impl Fn(&iced::Theme, button::Status) -> button::Style {
        |_theme, status| {
            let (background, border_color, shadow) = match status {
                button::Status::Active => (
                    CowboyTheme::glass_background(),
                    CowboyTheme::border_color(),
                    Shadow::default(),
                ),
                button::Status::Hovered => (
                    Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.15)),
                    CowboyTheme::border_hover_color(),
                    CowboyTheme::glow_shadow(),
                ),
                button::Status::Pressed => (
                    Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.12)),
                    CowboyTheme::border_hover_color(),
                    Shadow::default(),
                ),
                button::Status::Disabled => (
                    Background::Color(Color::from_rgba(0.5, 0.5, 0.5, 0.05)),
                    Color::from_rgba(0.5, 0.5, 0.5, 0.2),
                    Shadow::default(),
                ),
            };

            button::Style {
                background: Some(background),
                text_color: CowboyTheme::text_primary(),
                border: Border {
                    color: border_color,
                    width: 1.0,
                    radius: 10.0.into(),
                },
                shadow,
            }
        }
    }

    /// Security operation button (for key generation)
    pub fn security_button() -> impl Fn(&iced::Theme, button::Status) -> button::Style {
        |_theme, status| {
            let (background, shadow) = match status {
                button::Status::Active => (
                    CowboyTheme::security_gradient(),
                    CowboyTheme::button_shadow(),
                ),
                button::Status::Hovered => (
                    CowboyTheme::security_gradient(),
                    Shadow {
                        color: Color::from_rgba(0.0, 0.95, 0.99, 0.3),
                        offset: iced::Vector::new(0.0, 6.0),
                        blur_radius: 20.0,
                    },
                ),
                button::Status::Pressed => (
                    CowboyTheme::security_gradient(),
                    Shadow::default(),
                ),
                button::Status::Disabled => (
                    Background::Color(Color::from_rgba(0.5, 0.5, 0.5, 0.3)),
                    Shadow::default(),
                ),
            };

            button::Style {
                background: Some(background),
                text_color: CowboyTheme::text_primary(),
                border: Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: 12.0.into(),
                },
                shadow,
            }
        }
    }

    /// Glass morphism container style
    pub fn glass_container() -> impl Fn(&iced::Theme) -> container::Style {
        |_theme| container::Style {
            background: Some(CowboyTheme::glass_background()),
            text_color: Some(CowboyTheme::text_primary()),
            border: Border {
                color: CowboyTheme::border_color(),
                width: 1.0,
                radius: 20.0.into(),
            },
            shadow: CowboyTheme::glow_shadow(),
        }
    }

    /// Dark container style
    pub fn dark_container() -> impl Fn(&iced::Theme) -> container::Style {
        |_theme| container::Style {
            background: Some(CowboyTheme::dark_background()),
            text_color: Some(CowboyTheme::text_primary()),
            border: Border::default(),
            shadow: Shadow::default(),
        }
    }

    /// Card container style with enhanced shadow
    pub fn card_container() -> impl Fn(&iced::Theme) -> container::Style {
        |_theme| container::Style {
            background: Some(CowboyTheme::glass_dark_background()),
            text_color: Some(CowboyTheme::text_primary()),
            border: Border {
                color: CowboyTheme::border_color(),
                width: 1.0,
                radius: 15.0.into(),
            },
            // Enhanced shadow with gradient-like effect (50% opacity at center, fading to 0%)
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.5),  // Start at 50% opacity
                offset: iced::Vector::new(0.0, 8.0),
                blur_radius: 24.0,  // Large blur creates gradient fade to 0%
            },
        }
    }

    /// Card container with deeper shadow (for elevated cards)
    pub fn elevated_card_container() -> impl Fn(&iced::Theme) -> container::Style {
        |_theme| container::Style {
            background: Some(CowboyTheme::glass_dark_background()),
            text_color: Some(CowboyTheme::text_primary()),
            border: Border {
                color: CowboyTheme::border_hover_color(),
                width: 1.5,
                radius: 15.0.into(),
            },
            // Deeper shadow for elevated appearance
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.6),
                offset: iced::Vector::new(0.0, 12.0),
                blur_radius: 32.0,
            },
        }
    }

    /// Pastel teal card (colorful accent card)
    pub fn pastel_teal_card() -> impl Fn(&iced::Theme) -> container::Style {
        |_theme| container::Style {
            background: Some(CowboyTheme::pastel_teal_background()),
            text_color: Some(CowboyTheme::text_primary()),
            border: Border {
                color: Color::from_rgba(0.4, 0.7, 0.75, 0.6),
                width: 1.0,
                radius: 15.0.into(),
            },
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.4),
                offset: iced::Vector::new(0.0, 6.0),
                blur_radius: 20.0,
            },
        }
    }

    /// Pastel coral card (warm accent)
    pub fn pastel_coral_card() -> impl Fn(&iced::Theme) -> container::Style {
        |_theme| container::Style {
            background: Some(CowboyTheme::pastel_coral_background()),
            text_color: Some(CowboyTheme::text_primary()),
            border: Border {
                color: Color::from_rgba(0.95, 0.7, 0.6, 0.6),
                width: 1.0,
                radius: 15.0.into(),
            },
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.4),
                offset: iced::Vector::new(0.0, 6.0),
                blur_radius: 20.0,
            },
        }
    }

    /// Pastel cream card (neutral highlight)
    pub fn pastel_cream_card() -> impl Fn(&iced::Theme) -> container::Style {
        |_theme| container::Style {
            background: Some(CowboyTheme::pastel_cream_background()),
            text_color: Some(Color::from_rgb(0.2, 0.2, 0.3)),  // Darker text for cream bg
            border: Border {
                color: Color::from_rgba(0.95, 0.92, 0.85, 0.6),
                width: 1.0,
                radius: 15.0.into(),
            },
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.4),
                offset: iced::Vector::new(0.0, 6.0),
                blur_radius: 20.0,
            },
        }
    }

    /// Pastel mint card (fresh accent)
    pub fn pastel_mint_card() -> impl Fn(&iced::Theme) -> container::Style {
        |_theme| container::Style {
            background: Some(CowboyTheme::pastel_mint_background()),
            text_color: Some(CowboyTheme::text_primary()),
            border: Border {
                color: Color::from_rgba(0.7, 0.9, 0.8, 0.6),
                width: 1.0,
                radius: 15.0.into(),
            },
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.4),
                offset: iced::Vector::new(0.0, 6.0),
                blur_radius: 20.0,
            },
        }
    }

    /// Input field style
    pub fn glass_input() -> impl Fn(&iced::Theme, text_input::Status) -> text_input::Style {
        |theme, status| {
            let _base_style = text_input::default(theme, status);  // Reserved for style extension

            let (background, border_color) = match status {
                text_input::Status::Active => (
                    Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.08)),
                    CowboyTheme::border_color(),
                ),
                text_input::Status::Hovered => (
                    Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.12)),
                    CowboyTheme::border_hover_color(),
                ),
                text_input::Status::Focused => (
                    Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.15)),
                    Color::from_rgba(1.0, 1.0, 1.0, 0.5),
                ),
                text_input::Status::Disabled => (
                    Background::Color(Color::from_rgba(0.5, 0.5, 0.5, 0.05)),
                    Color::from_rgba(0.5, 0.5, 0.5, 0.2),
                ),
            };

            text_input::Style {
                background,
                border: Border {
                    color: border_color,
                    width: 1.0,
                    radius: 10.0.into(),
                },
                icon: CowboyTheme::text_secondary(),
                placeholder: CowboyTheme::text_muted(),
                value: CowboyTheme::text_primary(),
                selection: Color::from_rgba(0.165, 0.322, 0.596, 0.3),
            }
        }
    }

    /// Get the dark theme
    pub fn dark() -> iced::Theme {
        iced::Theme::custom("cowboy".into(), iced::theme::Palette {
            background: Color::from_rgb(0.039, 0.078, 0.157),
            text: Color::WHITE,
            primary: Color::from_rgb(0.165, 0.322, 0.596),
            success: Color::from_rgb(0.19, 0.81, 0.82),
            danger: Color::from_rgb(0.98, 0.44, 0.60),
        })
    }
}