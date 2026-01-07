<!-- Copyright (c) 2025 - Cowboy AI, LLC. -->

# Sprint 80 Retrospective: State Machine UI Integration

## Sprint Overview

**Duration**: 2026-01-07
**Status**: Completed
**Focus**: Wire state machine viewmodels to reflect current PKI and YubiKey state in GUI visualization

## What Was Accomplished

### Updated PKI Bootstrap State Machine Definition

Aligned `build_pki_bootstrap_state_machine()` to use exact state names matching the `PKIBootstrapState` enum:

```rust
// Before (generic names)
"NotStarted", "GeneratingRootCA", "GeneratingIntermediateCA", ...

// After (matches enum variants exactly)
"Uninitialized", "RootCAPlanned", "RootCAGenerated",
"IntermediateCAPlanned", "IntermediateCAGenerated",
"LeafCertsGenerated", "YubiKeysProvisioned",
"ExportReady", "Bootstrapped"
```

### Added YubiKey Provisioning State Machine Definition

Created `build_yubikey_provisioning_state_machine()` with exact state names matching `YubiKeyProvisioningState`:

```rust
"Detected" → "Authenticated" → "PINChanged" →
"ManagementKeyRotated" → "SlotPlanned" → "KeysGenerated" →
"CertificatesImported" → "Attested" → "Sealed"
```

### Added `state_name()` Helper Methods

Added methods to both enums to return the state name string for viewmodel integration:

```rust
impl PKIBootstrapState {
    pub fn state_name(&self) -> &'static str {
        match self {
            PKIBootstrapState::Uninitialized => "Uninitialized",
            PKIBootstrapState::RootCAPlanned { .. } => "RootCAPlanned",
            // ... all variants
        }
    }
}

impl YubiKeyProvisioningState {
    pub fn state_name(&self) -> &'static str {
        match self {
            YubiKeyProvisioningState::Detected { .. } => "Detected",
            // ... all variants
        }
    }
}
```

### Added Helper Functions for Current State

Created convenience functions that build state machine definitions with current state already set:

```rust
pub fn get_pki_bootstrap_with_current(
    pki_state: &PKIBootstrapState
) -> StateMachineDefinition {
    let current_state_name = pki_state.state_name();
    build_pki_bootstrap_state_machine().with_current(current_state_name)
}

pub fn get_yubikey_provisioning_with_current(
    yubikey_state: &YubiKeyProvisioningState
) -> StateMachineDefinition {
    let current_state_name = yubikey_state.state_name();
    build_yubikey_provisioning_state_machine().with_current(current_state_name)
}
```

### Wired GUI State Machine View

Updated three message handlers to use current state when viewing PKI/YubiKey state machines:

1. **StateMachineSelected** - When user selects a state machine type
2. **StateMachineMessage::SelectMachine** - When dropdown selection changes
3. **StateMachineMessage::ResetView** - When view is refreshed

```rust
Message::StateMachineSelected(sm_type) => {
    self.state_machine_definition = match sm_type {
        StateMachineType::PkiBootstrap => {
            get_pki_bootstrap_with_current(&self.pki_state)
        }
        StateMachineType::YubiKeyProvisioning => {
            if let Some(yk_state) = self.yubikey_states.values().next() {
                get_yubikey_provisioning_with_current(yk_state)
            } else {
                get_state_machine(sm_type)
            }
        }
        _ => get_state_machine(sm_type),
    };
    // Status shows current state
    let current_info = self.state_machine_definition.current_state
        .as_ref()
        .map(|c| format!(" (current: {})", c))
        .unwrap_or_default();
    self.status_message = format!("Viewing {} state machine{}", ...);
}
```

## UI Integration Flow

```
                     State Machine View Integration
┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│  User selects "PKI Bootstrap Workflow" from dropdown                │
│                      │                                              │
│                      ▼                                              │
│  StateMachineSelected(PkiBootstrap)                                 │
│                      │                                              │
│                      ▼                                              │
│  get_pki_bootstrap_with_current(&self.pki_state)                    │
│                      │                                              │
│                      ▼                                              │
│  pki_state.state_name() → "RootCAGenerated"                         │
│                      │                                              │
│                      ▼                                              │
│  StateMachineDefinition.with_current("RootCAGenerated")             │
│                      │                                              │
│                      ▼                                              │
│  Canvas renders with "RootCAGenerated" highlighted blue             │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

## Visual Rendering

The existing `StateMachineCanvas` already handles current state highlighting:

```rust
let fill_color = if is_current {
    Color::from_rgb(0.0, 0.5, 0.9)  // Blue highlight
} else if state.is_terminal {
    Color::from_rgb(0.6, 0.2, 0.2)  // Red
} else if state.is_initial {
    Color::from_rgb(0.2, 0.5, 0.3)  // Green
} else {
    Color::from_rgb(0.25, 0.35, 0.45)  // Gray
};
```

Legend shows:
- ◐ Initial State (green)
- ◉ Terminal State (red)
- ● Regular State (gray)
- ◎ Current State (blue highlight)

## Metrics

| Metric | Value |
|--------|-------|
| Lines added | ~180 |
| Tests passing | 1,072 |
| Files modified | 3 |
| New helper methods | 4 |
| Handlers updated | 3 |

## Files Modified

| File | Changes |
|------|---------|
| `src/gui/state_machine_graph.rs` | Updated PKI states, added YubiKey provisioning, added helper functions |
| `src/state_machines/workflows.rs` | Added `state_name()` methods to both enums |
| `src/gui.rs` | Updated 3 handlers to use current state |

## What Went Well

1. **Existing infrastructure** - StateMachineView and canvas rendering already supported `current_state`
2. **Clean separation** - Viewmodel pattern made integration straightforward
3. **State name matching** - Using exact enum variant names ensures reliable matching
4. **Backward compatible** - Non-workflow state machines continue working unchanged
5. **All tests pass** - No regressions

## Lessons Learned

1. **State name consistency is critical** - Mismatch between enum variant names and graph state names breaks highlighting
2. **Helper functions simplify integration** - `get_pki_bootstrap_with_current()` encapsulates the pattern
3. **Multiple YubiKeys need UX consideration** - Currently shows first YubiKey's state; could add selector

## Technical Notes

### State Name Extraction Pattern

The `state_name()` method returns a `&'static str` matching the enum variant:

```rust
pub fn state_name(&self) -> &'static str {
    match self {
        PKIBootstrapState::Uninitialized => "Uninitialized",
        PKIBootstrapState::RootCAPlanned { .. } => "RootCAPlanned",
        // ...
    }
}
```

This pattern:
- Returns static strings (no allocation)
- Ignores associated data (using `{ .. }`)
- Matches graph definition state names exactly

### Multi-YubiKey Handling

For YubiKeyProvisioning, we show the first YubiKey's state:

```rust
if let Some(yk_state) = self.yubikey_states.values().next() {
    get_yubikey_provisioning_with_current(yk_state)
} else {
    get_state_machine(sm_type)
}
```

Future enhancement could add a YubiKey serial selector dropdown.

## Related Sprints

- Sprint 77: Wired PKI handlers to PKIBootstrapState state machine (RootCA)
- Sprint 78: Wired full PKI chain (IntermediateCA, LeafCerts, YubiKey)
- Sprint 79: Wired Export workflow (ExportReady, Bootstrapped)
- Sprint 80: Wired UI viewmodel to reflect current state (this sprint)

## Next Steps

1. **YubiKey selector** - Add dropdown to choose which YubiKey's state to display
2. **State transition animation** - Animate when state changes
3. **State persistence** - Save PKI state to projection for recovery
4. **Actual crypto wiring** - Connect state machine to real operations
5. **Bounded context refactoring** - Extract domains from gui.rs per plan
