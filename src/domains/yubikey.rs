// Copyright (c) 2025 - Cowboy AI, LLC.

//! YubiKey Bounded Context
//!
//! This module defines the coproduct for the YubiKey hardware
//! bounded context, handling devices and PIV slots.
//!
//! ## Entities in this Context
//! - YubiKeyDevice (tier 0)
//! - PivSlot (tier 1)
//! - YubiKeyStatus (tier 0)

use std::fmt;
use uuid::Uuid;

use crate::domain::yubikey::{YubiKeyDevice, PivSlotView, YubiKeyStatus};

/// Injection tag for YubiKey bounded context
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum YubiKeyInjection {
    Device,
    Slot,
    Status,
}

impl YubiKeyInjection {
    /// Display name for this entity type
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Device => "YubiKey",
            Self::Slot => "PIV Slot",
            Self::Status => "YubiKey Status",
        }
    }

    /// Layout tier for hierarchical visualization
    pub fn layout_tier(&self) -> u8 {
        match self {
            Self::Device | Self::Status => 0,
            Self::Slot => 1,
        }
    }
}

impl fmt::Display for YubiKeyInjection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// Inner data for YubiKey context entities
#[derive(Debug, Clone)]
pub enum YubiKeyData {
    Device(YubiKeyDevice),
    Slot(PivSlotView),
    Status(YubiKeyStatus),
}

/// YubiKey Entity - Coproduct of YubiKey-related types
#[derive(Debug, Clone)]
pub struct YubiKeyEntity {
    injection: YubiKeyInjection,
    data: YubiKeyData,
}

impl YubiKeyEntity {
    // ========================================================================
    // Injection Functions
    // ========================================================================

    /// Inject YubiKey Device into coproduct
    pub fn inject_device(device: YubiKeyDevice) -> Self {
        Self {
            injection: YubiKeyInjection::Device,
            data: YubiKeyData::Device(device),
        }
    }

    /// Inject PIV Slot into coproduct
    pub fn inject_slot(slot: PivSlotView) -> Self {
        Self {
            injection: YubiKeyInjection::Slot,
            data: YubiKeyData::Slot(slot),
        }
    }

    /// Inject YubiKey Status into coproduct
    pub fn inject_status(status: YubiKeyStatus) -> Self {
        Self {
            injection: YubiKeyInjection::Status,
            data: YubiKeyData::Status(status),
        }
    }

    // ========================================================================
    // Accessors
    // ========================================================================

    /// Get the injection tag
    pub fn injection(&self) -> YubiKeyInjection {
        self.injection
    }

    /// Get reference to inner data
    pub fn data(&self) -> &YubiKeyData {
        &self.data
    }

    /// Get entity ID
    pub fn id(&self) -> Uuid {
        match &self.data {
            YubiKeyData::Device(d) => d.id.as_uuid(),
            YubiKeyData::Slot(s) => s.id.as_uuid(),
            YubiKeyData::Status(s) => s.person_id,
        }
    }

    /// Get entity name/label
    pub fn name(&self) -> String {
        match &self.data {
            YubiKeyData::Device(d) => d.serial.clone(),
            YubiKeyData::Slot(s) => s.slot_name.clone(),
            YubiKeyData::Status(s) => format!(
                "Status: {} of {} slots provisioned",
                s.slots_provisioned.len(),
                s.slots_provisioned.len() + s.slots_needed.len()
            ),
        }
    }

    // ========================================================================
    // Universal Property (Fold)
    // ========================================================================

    /// Apply a fold to this entity
    pub fn fold<F: FoldYubiKeyEntity>(&self, folder: &F) -> F::Output {
        match &self.data {
            YubiKeyData::Device(d) => folder.fold_device(d),
            YubiKeyData::Slot(s) => folder.fold_slot(s),
            YubiKeyData::Status(s) => folder.fold_status(s),
        }
    }
}

/// Universal property trait for YubiKeyEntity coproduct
pub trait FoldYubiKeyEntity {
    type Output;

    fn fold_device(&self, device: &YubiKeyDevice) -> Self::Output;
    fn fold_slot(&self, slot: &PivSlotView) -> Self::Output;
    fn fold_status(&self, status: &YubiKeyStatus) -> Self::Output;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct InjectionFolder;

    impl FoldYubiKeyEntity for InjectionFolder {
        type Output = YubiKeyInjection;

        fn fold_device(&self, _: &YubiKeyDevice) -> Self::Output {
            YubiKeyInjection::Device
        }
        fn fold_slot(&self, _: &PivSlotView) -> Self::Output {
            YubiKeyInjection::Slot
        }
        fn fold_status(&self, _: &YubiKeyStatus) -> Self::Output {
            YubiKeyInjection::Status
        }
    }
}
