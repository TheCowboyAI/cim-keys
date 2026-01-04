// Copyright (c) 2025 - Cowboy AI, LLC.

//! Workflow Visualization Component
//!
//! This module provides a UI component for visualizing trust chain gap
//! progression and navigating to the objects needed for fulfillment.
//!
//! ## Features
//!
//! - Progress bars for each gap
//! - Markov chain predictions for next steps
//! - Navigation to source files
//! - Evidence tracking (tests, BDD scenarios)

use iced::{
    widget::{
        button, column, container, row, text, progress_bar, scrollable,
        horizontal_space, vertical_space, Space, Column, Row,
    },
    Element, Length, Alignment,
};

use crate::workflow::{
    TrustChainGap, GapId, GapStatus, GapCategory,
    PredictedStep,
    ProgressionTracker,
    ObjectNavigator,
};

/// Messages for workflow view interaction
#[derive(Debug, Clone)]
pub enum WorkflowMessage {
    /// Gap was selected for viewing details
    GapSelected(GapId),
    /// Navigate to a specific target
    NavigateTo(String),
    /// Start working on a gap
    StartGap(GapId),
    /// Mark a gap as advanced
    AdvanceGap(GapId),
    /// Refresh predictions
    RefreshPredictions,
    /// Toggle expand/collapse for a gap
    ToggleGapExpanded(GapId),
}

/// State for the workflow view
pub struct WorkflowView {
    /// All gaps
    pub gaps: Vec<TrustChainGap>,
    /// Progression tracker
    pub tracker: ProgressionTracker,
    /// Navigator for file navigation
    pub navigator: ObjectNavigator,
    /// Currently selected gap
    pub selected_gap: Option<GapId>,
    /// Expanded gaps (showing details)
    pub expanded_gaps: Vec<GapId>,
    /// Current predictions
    pub predictions: Vec<PredictedStep>,
}

impl Default for WorkflowView {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkflowView {
    pub fn new() -> Self {
        let gaps = TrustChainGap::all_gaps();
        let tracker = ProgressionTracker::new(gaps.clone());
        let navigator = ObjectNavigator::new(".", gaps.clone());
        let predictions = tracker.recommend_next_n(3);

        Self {
            gaps,
            tracker,
            navigator,
            selected_gap: None,
            expanded_gaps: Vec::new(),
            predictions,
        }
    }

    /// Update the view based on a message
    pub fn update(&mut self, message: WorkflowMessage) {
        match message {
            WorkflowMessage::GapSelected(gap_id) => {
                self.selected_gap = Some(gap_id);
            }
            WorkflowMessage::NavigateTo(path) => {
                // Open file in editor (platform-specific)
                #[cfg(not(target_arch = "wasm32"))]
                {
                    let _ = std::process::Command::new("code")
                        .arg(&path)
                        .spawn();
                }
            }
            WorkflowMessage::StartGap(gap_id) => {
                let _ = self.tracker.start_gap(gap_id, None);
                self.refresh_predictions();
            }
            WorkflowMessage::AdvanceGap(gap_id) => {
                let _ = self.tracker.advance_gap(gap_id, "Manual advance".to_string());
                self.refresh_predictions();
            }
            WorkflowMessage::RefreshPredictions => {
                self.refresh_predictions();
            }
            WorkflowMessage::ToggleGapExpanded(gap_id) => {
                if self.expanded_gaps.contains(&gap_id) {
                    self.expanded_gaps.retain(|id| *id != gap_id);
                } else {
                    self.expanded_gaps.push(gap_id);
                }
            }
        }
    }

    fn refresh_predictions(&mut self) {
        self.predictions = self.tracker.recommend_next_n(3);
    }

    /// Render the workflow view
    pub fn view(&self) -> Element<'_, WorkflowMessage> {
        let summary = self.tracker.summary();

        // Header with overall progress
        let title = text("Trust Chain Gap Fulfillment")
            .size(24);

        let progress = progress_bar(0.0..=100.0, summary.completion_percentage as f32)
            .width(Length::Fixed(300.0))
            .height(20);

        let stats = text(format!(
            "{}/{} gaps complete ({:.0}%)",
            summary.completed,
            summary.total_gaps,
            summary.completion_percentage
        ))
        .size(14);

        let status_row = row![
            column![
                text("Completed").size(12),
                text(format!("{}", summary.completed)).size(20),
            ].align_x(Alignment::Center),
            horizontal_space().width(30),
            column![
                text("In Progress").size(12),
                text(format!("{}", summary.in_progress)).size(20),
            ].align_x(Alignment::Center),
            horizontal_space().width(30),
            column![
                text("Not Started").size(12),
                text(format!("{}", summary.not_started)).size(20),
            ].align_x(Alignment::Center),
        ]
        .align_y(Alignment::Center);

        let header = column![
            title,
            vertical_space().height(10),
            row![progress, horizontal_space().width(20), stats],
            vertical_space().height(10),
            status_row,
        ];

        // Predictions section
        let predictions_title = text("Recommended Next Steps")
            .size(18);

        let prediction_cards: Vec<Element<'_, WorkflowMessage>> = self.predictions
            .iter()
            .enumerate()
            .map(|(i, pred)| {
                let gap = self.gaps.iter().find(|g| g.id == pred.gap_id);
                let name = gap.map(|g| g.name.clone()).unwrap_or_else(|| "Unknown".to_string());
                let probability = format!("{:.0}%", pred.probability * 100.0);
                let effort = format!("Effort: {}/10", pred.expected_effort);

                let reasons_text: String = pred.reasoning
                    .iter()
                    .take(2)
                    .map(|r| format!("• {}", r))
                    .collect::<Vec<_>>()
                    .join("\n");

                column![
                    row![
                        text(format!("#{}", i + 1)).size(16),
                        horizontal_space().width(10),
                        text(probability).size(14),
                    ],
                    text(name).size(14),
                    text(effort).size(11),
                    text(reasons_text).size(11),
                    vertical_space().height(5),
                    button(text("Start").size(12))
                        .on_press(WorkflowMessage::StartGap(pred.gap_id))
                        .padding(5),
                ]
                .spacing(5)
                .padding(15)
                .width(Length::Fixed(250.0))
                .into()
            })
            .collect();

        let prediction_row = Row::with_children(prediction_cards)
            .spacing(15)
            .width(Length::Fill);

        let predictions_section = column![
            predictions_title,
            vertical_space().height(10),
            prediction_row,
        ];

        // Gap list
        let gap_items: Vec<Element<'_, WorkflowMessage>> = self.gaps
            .iter()
            .map(|gap| {
                let is_expanded = self.expanded_gaps.contains(&gap.id);
                let status = self.tracker.state().get_status(gap.id);

                let category_label = match gap.category {
                    GapCategory::Pki => "PKI",
                    GapCategory::Delegation => "DELEGATION",
                    GapCategory::YubiKey => "YUBIKEY",
                    GapCategory::Policy => "POLICY",
                    GapCategory::Domain => "DOMAIN",
                };

                let expand_label = if is_expanded { "▼" } else { "▶" };

                let gap_id = gap.id;
                let header_row = row![
                    button(text(expand_label).size(12))
                        .on_press(WorkflowMessage::ToggleGapExpanded(gap_id))
                        .padding(2),
                    horizontal_space().width(10),
                    text(category_label).size(10),
                    horizontal_space().width(10),
                    text(gap.name.clone()).size(14),
                    horizontal_space(),
                    text(format!("{:?}", status)).size(12),
                    horizontal_space().width(10),
                    progress_bar(0.0..=100.0, status.progress_percentage() as f32)
                        .width(Length::Fixed(150.0))
                        .height(8),
                    horizontal_space().width(10),
                    text(format!("P{}", gap.priority)).size(12),
                ]
                .align_y(Alignment::Center)
                .padding(10);

                let details: Element<'_, WorkflowMessage> = if is_expanded {
                    let action_button: Element<'_, WorkflowMessage> = match status {
                        GapStatus::NotStarted => {
                            button(text("Start Working").size(12))
                                .on_press(WorkflowMessage::StartGap(gap_id))
                                .padding(5)
                                .into()
                        }
                        GapStatus::Verified => {
                            text("✓ Complete").size(12).into()
                        }
                        _ => {
                            button(text("Advance").size(12))
                                .on_press(WorkflowMessage::AdvanceGap(gap_id))
                                .padding(5)
                                .into()
                        }
                    };

                    let deps_text = if gap.dependencies.is_empty() {
                        "No dependencies".to_string()
                    } else {
                        format!("Depends on: {:?}", gap.dependencies)
                    };

                    column![
                        text(gap.description.clone()).size(12),
                        vertical_space().height(10),
                        row![
                            text(format!("Unit: {}", gap.evidence.unit_tests)).size(11),
                            horizontal_space().width(15),
                            text(format!("Property: {}", gap.evidence.property_tests)).size(11),
                            horizontal_space().width(15),
                            text(format!("BDD: {}", gap.evidence.bdd_scenarios)).size(11),
                        ],
                        vertical_space().height(5),
                        text(deps_text).size(11),
                        vertical_space().height(10),
                        action_button,
                    ]
                    .padding(15)
                    .into()
                } else {
                    Space::new(0, 0).into()
                };

                column![header_row, details]
                    .into()
            })
            .collect();

        let gap_list = Column::with_children(gap_items)
            .spacing(10)
            .width(Length::Fill);

        let content = column![
            header,
            vertical_space().height(20),
            predictions_section,
            vertical_space().height(20),
            scrollable(gap_list).height(Length::Fill),
        ]
        .padding(20)
        .spacing(10);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
