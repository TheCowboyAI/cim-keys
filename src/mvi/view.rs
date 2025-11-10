//! View - Pure Rendering Function
//!
//! **Signature**: `Model → Element<Intent>`
//!
//! This module contains pure rendering functions that transform the Model
//! into visual UI elements. All user interactions produce Intent values.

use super::{Intent, Model};
use super::model::{Tab, DomainStatus, ExportStatus};
use iced::{
    widget::{button, column, container, row, text, text_input, progress_bar, scrollable, Column},
    Element, Length, Color, Border, Font, Theme, Background, Alignment,
};

/// Pure view function for MVI layer
///
/// **Design Principle**: This is a PURE function.
/// - Takes immutable Model reference
/// - Returns visual representation as Element<Intent>
/// - NO side effects, NO state mutation
/// - All interactions produce Intent values
pub fn view(model: &Model) -> Element<'_, Intent> {
    let content = match model.current_tab {
        Tab::Welcome => view_welcome(model),
        Tab::Organization => view_organization(model),
        Tab::Keys => view_keys(model),
        Tab::Export => view_export(model),
    };

    column![
        view_header(model),
        view_error_banner(model),
        content,
        view_footer(model),
    ]
    .spacing(10)
    .padding(20)
    .into()
}

/// Header with logo and tab navigation
fn view_header(model: &Model) -> Element<'_, Intent> {
    let logo = container(
        column![
            text("CIM").size(32).font(Font {
                family: iced::font::Family::Monospace,
                weight: iced::font::Weight::Bold,
                ..Default::default()
            }),
            text("KEYS").size(24).font(Font {
                family: iced::font::Family::Monospace,
                weight: iced::font::Weight::Bold,
                ..Default::default()
            }),
        ]
        .align_x(Alignment::Center)
        .spacing(0)
    )
    .width(Length::Fixed(100.0))
    .height(Length::Fixed(80.0))
    .center(Length::Fixed(80.0))
    .style(|theme: &Theme| {
        let palette = theme.extended_palette();
        container::Style {
            background: Some(Background::Color(Color::from_rgba(0.1, 0.1, 0.2, 1.0))),
            border: Border {
                color: palette.primary.strong.color,
                width: 2.0,
                radius: 8.0.into(),
            },
            text_color: Some(palette.primary.strong.color),
            ..container::Style::default()
        }
    });

    let tab_bar = row![
        tab_button("Welcome", Tab::Welcome, model.current_tab == Tab::Welcome),
        tab_button("Organization", Tab::Organization, model.current_tab == Tab::Organization),
        tab_button("Keys", Tab::Keys, model.current_tab == Tab::Keys),
        tab_button("Export", Tab::Export, model.current_tab == Tab::Export),
    ]
    .spacing(10);

    row![
        logo,
        container(tab_bar)
            .width(Length::Fill)
            .center(Length::Fill)
    ]
    .spacing(20)
    .align_y(Alignment::Center)
    .into()
}

/// Helper to create tab buttons
fn tab_button(label: &str, tab: Tab, is_active: bool) -> Element<'_, Intent> {
    let btn = button(text(label).size(14))
        .on_press(Intent::UiTabSelected(tab));

    if is_active {
        btn.style(|theme: &Theme, _| {
            let palette = theme.extended_palette();
            button::Style {
                background: Some(Background::Color(palette.primary.strong.color)),
                text_color: palette.primary.strong.text,
                border: Border {
                    color: palette.primary.strong.color,
                    width: 2.0,
                    radius: 4.0.into(),
                },
                ..button::Style::default()
            }
        })
    } else {
        btn.style(|theme: &Theme, _| {
            let palette = theme.extended_palette();
            button::Style {
                background: Some(Background::Color(Color::TRANSPARENT)),
                text_color: palette.primary.base.color,
                border: Border {
                    color: palette.primary.weak.color,
                    width: 1.0,
                    radius: 4.0.into(),
                },
                ..button::Style::default()
            }
        })
    }.into()
}

/// Error banner (if error exists)
fn view_error_banner(model: &Model) -> Element<'_, Intent> {
    if let Some(ref error) = model.error_message {
        container(
            row![
                text("⚠ Error: ").size(14),
                text(error).size(14),
                button(text("✕").size(14))
                    .on_press(Intent::ErrorDismissed {
                        error_id: "current".to_string()
                    })
            ]
            .spacing(10)
            .padding(10)
        )
        .width(Length::Fill)
        .style(|_theme: &Theme| {
            container::Style {
                background: Some(Background::Color(Color::from_rgb(0.8, 0.2, 0.2))),
                text_color: Some(Color::WHITE),
                border: Border {
                    radius: 4.0.into(),
                    ..Default::default()
                },
                ..container::Style::default()
            }
        })
        .into()
    } else {
        row![].into()
    }
}

/// Footer with status message
fn view_footer(model: &Model) -> Element<'_, Intent> {
    container(
        text(&model.status_message).size(12)
    )
    .width(Length::Fill)
    .padding(10)
    .style(|theme: &Theme| {
        let palette = theme.extended_palette();
        container::Style {
            background: Some(Background::Color(palette.background.weak.color)),
            text_color: Some(palette.background.weak.text),
            ..container::Style::default()
        }
    })
    .into()
}

/// Welcome tab view
fn view_welcome(model: &Model) -> Element<'_, Intent> {
    let welcome_text = column![
        text("🔐 CIM Keys - Offline Domain Bootstrap").size(28),
        text("Secure cryptographic key management for CIM infrastructure").size(16),
        text("").size(10),
        text("This tool generates:").size(14),
        text("  • Root CA and certificate hierarchy").size(12),
        text("  • SSH keys for all users").size(12),
        text("  • GPG keys for secure communication").size(12),
        text("  • YubiKey provisioning").size(12),
        text("  • NATS operator/account/user credentials").size(12),
        text("").size(10),
        text("⚠️  Ensure this computer is air-gapped!").size(14)
            .color(Color::from_rgb(0.9, 0.5, 0.0)),
    ]
    .spacing(10)
    .padding(20);

    let actions = column![
        button(text("Create New Domain").size(16))
            .width(Length::Fixed(250.0))
            .on_press(Intent::UiCreateDomainClicked),
        button(text("Load Existing Domain").size(16))
            .width(Length::Fixed(250.0))
            .on_press(Intent::UiLoadDomainClicked {
                path: model.output_directory.clone()
            }),
    ]
    .spacing(15)
    .padding(20);

    column![
        welcome_text,
        actions,
    ]
    .spacing(30)
    .padding(40)
    .into()
}

/// Organization tab view
fn view_organization(model: &Model) -> Element<'_, Intent> {
    let domain_info = match &model.domain_status {
        DomainStatus::NotCreated => {
            column![
                text("No domain created yet").size(16),
                text("Go to Welcome tab to create a domain").size(12),
            ]
        }
        DomainStatus::Creating => {
            column![
                text("Creating domain...").size(16),
                progress_bar(0.0..=1.0, 0.5),
            ]
        }
        DomainStatus::Created { organization_id, organization_name } => {
            column![
                text(format!("Organization: {}", organization_name)).size(18),
                text(format!("ID: {}", organization_id)).size(12),
            ]
        }
        DomainStatus::LoadError(err) => {
            column![
                text("Failed to load domain").size(16).color(Color::from_rgb(0.8, 0.2, 0.2)),
                text(err).size(12),
            ]
        }
    };

    let org_form = column![
        text("Organization Details").size(16),
        text_input("Organization Name", &model.organization_name)
            .on_input(Intent::UiOrganizationNameChanged),
        text_input("Organization ID", &model.organization_id)
            .on_input(Intent::UiOrganizationIdChanged),
    ]
    .spacing(10)
    .padding(10);

    let mut people_list = Column::new().spacing(10).padding(10);
    people_list = people_list.push(text("People").size(16));

    for (index, person) in model.people.iter().enumerate() {
        let person_row = row![
            text_input("Name", &person.name)
                .width(Length::FillPortion(2))
                .on_input(move |name| Intent::UiPersonNameChanged { index, name }),
            text_input("Email", &person.email)
                .width(Length::FillPortion(2))
                .on_input(move |email| Intent::UiPersonEmailChanged { index, email }),
            button(text("✕").size(14))
                .on_press(Intent::UiRemovePersonClicked { index }),
        ]
        .spacing(10)
        .align_y(Alignment::Center);

        people_list = people_list.push(person_row);
    }

    people_list = people_list.push(
        button(text("+ Add Person").size(14))
            .on_press(Intent::UiAddPersonClicked)
    );

    scrollable(
        column![
            domain_info,
            org_form,
            people_list,
        ]
        .spacing(20)
        .padding(20)
    )
    .into()
}

/// Keys tab view
fn view_keys(model: &Model) -> Element<'_, Intent> {
    // STEP 1: Master Passphrase Entry
    let passphrase_section = {
        let strength_text = if let Some(strength) = model.passphrase_strength {
            format!("Strength: {}", strength.description())
        } else {
            "Enter passphrase to see strength".to_string()
        };

        let strength_color = model.passphrase_strength.map_or(
            Color::from_rgb(0.5, 0.5, 0.5),
            |s| match s {
                crate::crypto::passphrase::PassphraseStrength::TooWeak => Color::from_rgb(0.8, 0.2, 0.2),
                crate::crypto::passphrase::PassphraseStrength::Weak => Color::from_rgb(0.9, 0.5, 0.2),
                crate::crypto::passphrase::PassphraseStrength::Moderate => Color::from_rgb(0.9, 0.8, 0.2),
                crate::crypto::passphrase::PassphraseStrength::Strong => Color::from_rgb(0.3, 0.8, 0.3),
                crate::crypto::passphrase::PassphraseStrength::VeryStrong => Color::from_rgb(0.2, 0.9, 0.3),
            }
        );

        let seed_status = if model.master_seed_derived {
            column![
                text("✓ Master Seed Derived").size(14).color(Color::from_rgb(0.3, 0.8, 0.3)),
                text("All keys will be deterministically generated from this seed").size(11),
            ]
            .spacing(5)
        } else {
            column![
                text("Master seed not yet derived").size(14).color(Color::from_rgb(0.7, 0.7, 0.7)),
                text("Enter passphrase to derive cryptographic seed").size(11),
            ]
            .spacing(5)
        };

        let passphrase_match = if !model.passphrase.is_empty() && !model.passphrase_confirmed.is_empty() {
            if model.passphrase == model.passphrase_confirmed {
                text("✓ Passphrases match").size(11).color(Color::from_rgb(0.3, 0.8, 0.3))
            } else {
                text("✗ Passphrases do not match").size(11).color(Color::from_rgb(0.8, 0.2, 0.2))
            }
        } else {
            text("").size(11)
        };

        let derive_button = if model.master_seed_derived {
            button(text("Re-derive Master Seed").size(14))
                .width(Length::Fixed(250.0))
        } else {
            button(text("Derive Master Seed").size(14))
                .width(Length::Fixed(250.0))
                .on_press(Intent::UiDeriveMasterSeedClicked)
        };

        column![
            text("Step 1: Master Passphrase").size(18),
            text("All keys are derived from a single master passphrase using Argon2id (1GB memory, 10 iterations)").size(11),
            text(""),
            text("Master Passphrase:").size(12),
            text_input("Enter master passphrase (minimum 4 words or 20 characters)", &model.passphrase)
                .on_input(Intent::UiPassphraseChanged)
                .width(Length::Fixed(400.0)),
            text(strength_text).size(12).color(strength_color),
            text(""),
            text("Confirm Passphrase:").size(12),
            text_input("Re-enter passphrase", &model.passphrase_confirmed)
                .on_input(Intent::UiPassphraseConfirmChanged)
                .width(Length::Fixed(400.0)),
            passphrase_match,
            text(""),
            derive_button,
            text(""),
            seed_status,
        ]
        .spacing(5)
        .padding(20)
    };

    let mut key_status_items = vec![
        text("Key Generation Status").size(18).into(),
        text(format!(
            "Root CA: {}",
            if model.key_generation_status.root_ca_generated { "✓ Generated" } else { "Not generated" }
        )).size(14).into(),
    ];

    // Add certificate details if Root CA was generated
    if let Some(fingerprint) = &model.key_generation_status.root_ca_fingerprint {
        key_status_items.push(
            text(format!("  Fingerprint: {}", fingerprint))
                .size(12)
                .into()
        );
    }

    if let Some(cert_pem) = &model.key_generation_status.root_ca_certificate_pem {
        let lines = cert_pem.lines().count();
        key_status_items.push(
            text(format!("  Certificate: {} lines", lines))
                .size(12)
                .into()
        );
    }

    key_status_items.push(
        text(format!(
            "SSH Keys: {} generated",
            model.key_generation_status.ssh_keys_generated.len()
        )).size(14).into()
    );
    key_status_items.push(
        text(format!(
            "YubiKeys Provisioned: {}",
            model.key_generation_status.yubikeys_provisioned.len()
        )).size(14).into()
    );

    let key_status = Column::with_children(key_status_items)
        .spacing(10)
        .padding(20);

    let progress = column![
        text("Generation Progress").size(16),
        progress_bar(0.0..=1.0, model.key_generation_progress),
        text(format!("{:.0}%", model.key_generation_progress * 100.0)).size(12),
    ]
    .spacing(10)
    .padding(20);

    let actions = column![
        button(text("Generate Root CA").size(16))
            .width(Length::Fixed(250.0))
            .on_press(Intent::UiGenerateRootCAClicked),
        button(text("Generate SSH Keys").size(16))
            .width(Length::Fixed(250.0))
            .on_press(Intent::UiGenerateSSHKeysClicked),
        button(text("Generate All Keys").size(16))
            .width(Length::Fixed(250.0))
            .on_press(Intent::UiGenerateAllKeysClicked),
    ]
    .spacing(15)
    .padding(20);

    let mut yubikey_provisioning = Column::new().spacing(10).padding(20);
    yubikey_provisioning = yubikey_provisioning.push(text("YubiKey Provisioning").size(16));

    for (index, person) in model.people.iter().enumerate() {
        let is_provisioned = model.key_generation_status.yubikeys_provisioned.contains(&person.id);

        let person_row = row![
            text(&person.name).width(Length::FillPortion(2)),
            text(if is_provisioned { "✓ Provisioned" } else { "Not provisioned" })
                .size(12)
                .width(Length::FillPortion(1)),
            button(text("Provision YubiKey").size(12))
                .on_press(Intent::UiProvisionYubiKeyClicked { person_index: index }),
        ]
        .spacing(10)
        .align_y(Alignment::Center);

        yubikey_provisioning = yubikey_provisioning.push(person_row);
    }

    scrollable(
        column![
            passphrase_section,
            container(text("")).height(Length::Fixed(20.0)), // Spacer
            key_status,
            progress,
            actions,
            yubikey_provisioning,
        ]
        .spacing(20)
    )
    .into()
}

/// Export tab view
fn view_export(model: &Model) -> Element<'_, Intent> {
    let export_status_text = match &model.export_status {
        ExportStatus::NotStarted => "Ready to export".to_string(),
        ExportStatus::InProgress => "Exporting...".to_string(),
        ExportStatus::Completed { path, bytes_written } => {
            format!("✓ Exported {} bytes to {}", bytes_written, path.display())
        }
        ExportStatus::Failed { error } => {
            format!("✗ Export failed: {}", error)
        }
    };

    let status = column![
        text("Export Status").size(18),
        text(export_status_text).size(14),
    ]
    .spacing(10)
    .padding(20);

    let config = column![
        text("Export Configuration").size(16),
        text(format!("Output Directory: {}", model.output_directory.display())).size(12),
        text("").size(10),
        text("The following will be exported:").size(14),
        text("  • Domain configuration (organization, people)").size(12),
        text("  • Generated certificates and keys").size(12),
        text("  • NATS credentials").size(12),
        text("  • Audit log of all operations").size(12),
    ]
    .spacing(10)
    .padding(20);

    let actions = column![
        button(text("Export to SD Card").size(16))
            .width(Length::Fixed(250.0))
            .on_press(Intent::UiExportClicked {
                output_path: model.output_directory.clone()
            }),
    ]
    .spacing(15)
    .padding(20);

    scrollable(
        column![
            status,
            config,
            actions,
        ]
        .spacing(20)
    )
    .into()
}
