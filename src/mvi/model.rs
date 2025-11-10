//! Model - Pure Immutable State
//!
//! The Model contains ONLY UI state and display projections.
//! NO communication state, NO port instances, NO async operations.

use std::path::PathBuf;

/// Pure Display Model for MVI Layer
#[derive(Debug, Clone)]
pub struct Model {
    // ===== UI Navigation State =====
    pub current_tab: Tab,

    // ===== Domain Bootstrap State =====
    pub organization_name: String,
    pub organization_id: String,
    pub people: Vec<PersonInput>,

    // ===== Passphrase & Crypto State =====
    pub passphrase: String,
    pub passphrase_confirmed: String,
    pub passphrase_strength: Option<crate::crypto::passphrase::PassphraseStrength>,
    pub master_seed_derived: bool,
    pub master_seed: Option<crate::crypto::MasterSeed>,

    // ===== Display Projections =====
    pub domain_status: DomainStatus,
    pub key_generation_status: KeyGenerationStatus,
    pub export_status: ExportStatus,

    // ===== UI Feedback State =====
    pub status_message: String,
    pub error_message: Option<String>,
    pub key_generation_progress: f32,

    // ===== Output Configuration =====
    pub output_directory: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Welcome,
    Organization,
    Keys,
    Export,
}

#[derive(Debug, Clone)]
pub struct PersonInput {
    pub name: String,
    pub email: String,
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainStatus {
    NotCreated,
    Creating,
    Created {
        organization_id: String,
        organization_name: String,
    },
    LoadError(String),
}

#[derive(Debug, Clone)]
pub struct KeyGenerationStatus {
    pub root_ca_generated: bool,
    pub root_ca_certificate_pem: Option<String>,
    pub root_ca_fingerprint: Option<String>,
    pub ssh_keys_generated: Vec<String>, // person IDs
    pub yubikeys_provisioned: Vec<String>, // person IDs
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExportStatus {
    NotStarted,
    InProgress,
    Completed { path: PathBuf, bytes_written: usize },
    Failed { error: String },
}

impl Default for Model {
    fn default() -> Self {
        Self {
            current_tab: Tab::Welcome,
            organization_name: String::new(),
            organization_id: String::new(),
            people: Vec::new(),
            passphrase: String::new(),
            passphrase_confirmed: String::new(),
            passphrase_strength: None,
            master_seed_derived: false,
            master_seed: None,
            domain_status: DomainStatus::NotCreated,
            key_generation_status: KeyGenerationStatus {
                root_ca_generated: false,
                root_ca_certificate_pem: None,
                root_ca_fingerprint: None,
                ssh_keys_generated: Vec::new(),
                yubikeys_provisioned: Vec::new(),
            },
            export_status: ExportStatus::NotStarted,
            status_message: "Welcome to CIM Keys".to_string(),
            error_message: None,
            key_generation_progress: 0.0,
            output_directory: PathBuf::from("/tmp/cim-keys-output"),
        }
    }
}

impl Model {
    /// Create a new model with output directory
    pub fn new(output_directory: PathBuf) -> Self {
        Self {
            output_directory,
            ..Default::default()
        }
    }

    /// Update model with new tab (pure function)
    pub fn with_tab(mut self, tab: Tab) -> Self {
        self.current_tab = tab;
        self
    }

    /// Update model with new organization name (pure function)
    pub fn with_organization_name(mut self, name: String) -> Self {
        self.organization_name = name;
        self
    }

    /// Update model with new organization ID (pure function)
    pub fn with_organization_id(mut self, id: String) -> Self {
        self.organization_id = id;
        self
    }

    /// Add a person to the model (pure function)
    pub fn with_person_added(mut self, person: PersonInput) -> Self {
        self.people.push(person);
        self
    }

    /// Remove a person at index (pure function)
    pub fn with_person_removed(mut self, index: usize) -> Self {
        if index < self.people.len() {
            self.people.remove(index);
        }
        self
    }

    /// Update person name at index (pure function)
    pub fn with_person_name_updated(mut self, index: usize, name: String) -> Self {
        if let Some(person) = self.people.get_mut(index) {
            person.name = name;
        }
        self
    }

    /// Update person email at index (pure function)
    pub fn with_person_email_updated(mut self, index: usize, email: String) -> Self {
        if let Some(person) = self.people.get_mut(index) {
            person.email = email;
        }
        self
    }

    /// Update domain status (pure function)
    pub fn with_domain_status(mut self, status: DomainStatus) -> Self {
        self.domain_status = status;
        self
    }

    /// Update status message (pure function)
    pub fn with_status_message(mut self, message: String) -> Self {
        self.status_message = message;
        self
    }

    /// Update error message (pure function)
    pub fn with_error(mut self, error: Option<String>) -> Self {
        self.error_message = error;
        self
    }

    /// Update key generation progress (pure function)
    pub fn with_key_progress(mut self, progress: f32) -> Self {
        self.key_generation_progress = progress;
        self
    }

    /// Mark root CA as generated (pure function)
    pub fn with_root_ca_generated(mut self) -> Self {
        self.key_generation_status.root_ca_generated = true;
        self
    }

    /// Store root CA certificate data (pure function)
    pub fn with_root_ca_certificate(mut self, certificate_pem: String, fingerprint: String) -> Self {
        self.key_generation_status.root_ca_generated = true;
        self.key_generation_status.root_ca_certificate_pem = Some(certificate_pem);
        self.key_generation_status.root_ca_fingerprint = Some(fingerprint);
        self
    }

    /// Add SSH key generated for person (pure function)
    pub fn with_ssh_key_generated(mut self, person_id: String) -> Self {
        self.key_generation_status.ssh_keys_generated.push(person_id);
        self
    }

    /// Add YubiKey provisioned for person (pure function)
    pub fn with_yubikey_provisioned(mut self, person_id: String) -> Self {
        self.key_generation_status.yubikeys_provisioned.push(person_id);
        self
    }

    /// Update export status (pure function)
    pub fn with_export_status(mut self, status: ExportStatus) -> Self {
        self.export_status = status;
        self
    }

    /// Update passphrase (pure function)
    pub fn with_passphrase(mut self, passphrase: String) -> Self {
        self.passphrase = passphrase;
        self
    }

    /// Update passphrase confirmation (pure function)
    pub fn with_passphrase_confirmed(mut self, confirmed: String) -> Self {
        self.passphrase_confirmed = confirmed;
        self
    }

    /// Update passphrase strength (pure function)
    pub fn with_passphrase_strength(
        mut self,
        strength: Option<crate::crypto::passphrase::PassphraseStrength>,
    ) -> Self {
        self.passphrase_strength = strength;
        self
    }

    /// Mark master seed as derived (pure function)
    pub fn with_master_seed_derived(mut self, derived: bool) -> Self {
        self.master_seed_derived = derived;
        self
    }

    /// Store the master seed securely (pure function)
    pub fn with_master_seed(mut self, seed: crate::crypto::MasterSeed) -> Self {
        self.master_seed = Some(seed);
        self.master_seed_derived = true;
        self
    }

    /// Clear the master seed from memory (pure function)
    pub fn without_master_seed(mut self) -> Self {
        self.master_seed = None;
        self.master_seed_derived = false;
        self
    }
}

impl PersonInput {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            email: String::new(),
            id: uuid::Uuid::now_v7().to_string(),
        }
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = name;
        self
    }

    pub fn with_email(mut self, email: String) -> Self {
        self.email = email;
        self
    }
}

impl Default for PersonInput {
    fn default() -> Self {
        Self::new()
    }
}
