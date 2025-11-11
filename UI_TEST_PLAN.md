# CIM Keys GUI - Comprehensive UI Test Plan

## Test Environment
- **Application**: cim-keys GUI
- **Framework**: Iced 0.13+ (native and WASM compatible)
- **Architecture**: MVI (Model-View-Intent) with Event Sourcing
- **Date**: 2025-11-11

## Overview
This document provides a comprehensive test plan for the cim-keys GUI application, which manages cryptographic keys and PKI for CIM infrastructure in an offline, air-gapped environment.

## Application Structure

### Tabs/Screens
1. **Welcome Tab** - Initial landing page and domain configuration
2. **Organization Tab** - Organization graph visualization and people management
3. **Keys Tab** - Key generation and PKI hierarchy management
4. **Export Tab** - Export domain configuration to encrypted storage

### Key Features
- Offline-first architecture (air-gapped operation)
- Event-sourced state management
- Graph visualization of organization structure
- X.509 PKI hierarchy generation
- SSH key generation
- YubiKey integration (when available)
- Animated firefly shader background
- Cowboy AI theme with glassmorphism effects

## Test Categories

---

## 1. Welcome Tab Tests

### 1.1 Initial State
- [ ] Application launches successfully
- [ ] Welcome message is displayed
- [ ] Security notice is visible
- [ ] Organization name input field is present
- [ ] Domain input field is present
- [ ] "Load Existing Domain" button is present
- [ ] "Create New Domain" button is present

### 1.2 Create New Domain Flow
- [ ] Enter organization name (e.g., "TheCowboyAI")
- [ ] Enter domain (e.g., "cowboyai.com")
- [ ] Click "Create New Domain"
- [ ] Application transitions to Organization tab
- [ ] Status message updates appropriately
- [ ] Domain information is retained

### 1.3 Load Existing Domain Flow
- [ ] Click "Load Existing Domain"
- [ ] File picker dialog opens (native only)
- [ ] Select valid bootstrap JSON file
- [ ] Domain configuration loads successfully
- [ ] People are populated in organization graph
- [ ] Application transitions to Organization tab
- [ ] Status message shows number of people loaded

### 1.4 Form Validation
- [ ] Empty organization name is handled gracefully
- [ ] Empty domain is handled gracefully
- [ ] Invalid characters in inputs are handled
- [ ] Long organization names display correctly

---

## 2. Organization Tab Tests

### 2.1 Initial Layout
- [ ] "Organization Structure" title is displayed
- [ ] "Add Person" form is visible
- [ ] Graph canvas is rendered
- [ ] Graph has light gray background
- [ ] Graph has border

### 2.2 Add Person Form
- [ ] Name input field accepts text
- [ ] Email input field accepts text
- [ ] Role dropdown shows all role options:
  - RootAuthority
  - SecurityAdmin
  - Developer
  - ServiceAccount
  - BackupHolder
  - Auditor
- [ ] Role can be selected from dropdown
- [ ] "Add Person" button is visible

### 2.3 Add Person Functionality
- [ ] Enter name: "John Doe"
- [ ] Enter email: "john@example.com"
- [ ] Select role: "Developer"
- [ ] Click "Add Person"
- [ ] Person appears as node in graph
- [ ] Form fields are cleared after adding
- [ ] Status message confirms addition
- [ ] Person ID is generated (UUID v7)

### 2.4 Add Multiple People
- [ ] Add person with role "RootAuthority"
- [ ] Add person with role "SecurityAdmin"
- [ ] Add person with role "Developer"
- [ ] All three nodes appear in graph
- [ ] Nodes do not overlap
- [ ] Each node shows person name

### 2.5 Form Validation
- [ ] Adding person with empty name shows error
- [ ] Adding person with empty email shows error
- [ ] Error message displays in red banner
- [ ] Error can be dismissed with ✕ button

### 2.6 Graph Visualization
- [ ] Nodes are rendered as circles
- [ ] Node colors differ by role
- [ ] Person names are displayed below nodes
- [ ] Graph is responsive to window size

### 2.7 Graph Interactions
- [ ] Click on node selects person
- [ ] Selected node is highlighted
- [ ] Status message shows "Selected person in graph"
- [ ] Selected person ID is tracked

### 2.8 Load Existing Domain
- [ ] Load bootstrap config with 5+ people
- [ ] All people render as nodes
- [ ] Graph auto-layouts nodes
- [ ] No node overlap occurs
- [ ] Status message shows count

---

## 3. Keys Tab Tests

### 3.1 Initial Layout
- [ ] "Generate Keys for Organization" title displayed
- [ ] "PKI Hierarchy Generation" section visible
- [ ] Four numbered sections:
  1. Root CA
  2. Intermediate CA
  3. Server Certificates
  4. Other Keys
- [ ] All generation buttons are enabled

### 3.2 Root CA Generation
- [ ] Click "Generate Root CA" button
- [ ] Status message updates: "Generating Root CA certificate..."
- [ ] Progress bar appears
- [ ] Progress shows 10%
- [ ] Generation completes successfully
- [ ] Status message shows certificate ID
- [ ] Certificate saved to output directory

### 3.3 Intermediate CA Generation (MVI Pattern)
- [ ] Enter CA name: "Engineering"
- [ ] Click "Generate Intermediate CA"
- [ ] Status message updates with CA name
- [ ] Progress bar shows 20%
- [ ] MVI intent is processed
- [ ] Certificate appears in "Generated Certificates" list
- [ ] Shows: ✓ CA: Engineering - [fingerprint]
- [ ] Certificate has green checkmark
- [ ] Fingerprint is displayed (first 16 chars)

### 3.4 Multiple Intermediate CAs
- [ ] Generate "Engineering" CA
- [ ] Generate "Operations" CA
- [ ] Generate "Security" CA
- [ ] All three appear in list
- [ ] Each has unique fingerprint
- [ ] List is scrollable if needed

### 3.5 Server Certificate Generation (MVI Pattern)
- [ ] Enter Common Name: "nats.example.com"
- [ ] Enter SANs: "nats1.example.com, 10.0.0.1"
- [ ] Select intermediate CA: "Engineering"
- [ ] Click "Generate Server Certificate"
- [ ] Status message updates
- [ ] Progress bar shows 30%
- [ ] Certificate appears in "Generated Certificates" list
- [ ] Shows server cert with CN and signing CA
- [ ] Fingerprint is displayed

### 3.6 Server Certificate Validation
- [ ] Try to generate cert without selecting CA
- [ ] Error message displayed: "Please select an intermediate CA first"
- [ ] Error can be dismissed
- [ ] Generation is blocked

### 3.7 SAN Parsing
- [ ] Enter SANs: "host1.com, host2.com, 192.168.1.1"
- [ ] SANs are split by comma
- [ ] Whitespace is trimmed
- [ ] Empty entries are filtered

### 3.8 SSH Key Generation
- [ ] Click "Generate SSH Keys for All"
- [ ] Status message updates
- [ ] Progress bar shows 30%
- [ ] (TODO: Implementation pending)

### 3.9 YubiKey Provisioning
- [ ] Click "Provision YubiKeys"
- [ ] Status message updates
- [ ] (TODO: Implementation pending)

### 3.10 Generate All Keys
- [ ] Add 3 people in Organization tab
- [ ] Click "Generate All Keys"
- [ ] Event emitter creates command
- [ ] Correlation ID is generated
- [ ] Commands are drained and processed
- [ ] Aggregate handles commands
- [ ] Events are generated
- [ ] Projection is updated
- [ ] Keys count increments
- [ ] Progress bar reaches 100%
- [ ] Application transitions to Export tab

### 3.11 Progress Display
- [ ] Progress bar animates smoothly
- [ ] Percentage text updates (0-100%)
- [ ] "X of Y keys generated" message shown
- [ ] Progress persists during generation

### 3.12 Certificate Display
- [ ] Generated certificates list scrolls
- [ ] Intermediate CAs show ✓ checkmark
- [ ] Server certs show ✓ checkmark
- [ ] Fingerprints are abbreviated
- [ ] Signing CA name is shown for server certs
- [ ] Colors (green for success) are visible

---

## 4. Export Tab Tests

### 4.1 Initial Layout
- [ ] "Export Domain Configuration" title displayed
- [ ] "Export Options" section visible
- [ ] Output directory input shows current path
- [ ] All checkboxes are visible:
  - Include public keys (default: checked)
  - Include certificates (default: checked)
  - Generate NATS configuration (default: checked)
  - Include private keys (default: unchecked)
- [ ] "Export to Encrypted SD Card" button visible

### 4.2 Export Options
- [ ] Check "Include public keys"
- [ ] State updates correctly
- [ ] Uncheck "Include public keys"
- [ ] State updates correctly

### 4.3 Private Key Export
- [ ] Check "Include private keys" checkbox
- [ ] Password input field appears
- [ ] Password input is secure (masked)
- [ ] Uncheck "Include private keys"
- [ ] Password field disappears

### 4.4 Export Path Configuration
- [ ] Enter custom path: "/tmp/cim-export"
- [ ] Path is updated in state
- [ ] Long paths display correctly
- [ ] Invalid paths are handled

### 4.5 Export Execution
- [ ] Configure export options
- [ ] Click "Export to Encrypted SD Card"
- [ ] Projection is accessed
- [ ] Manifest is saved
- [ ] Status message shows export path
- [ ] Success confirmation displayed

### 4.6 Export Error Handling
- [ ] Export to read-only location
- [ ] Error message displayed
- [ ] Status shows failure reason
- [ ] Application remains responsive

---

## 5. Navigation Tests

### 5.1 Tab Switching
- [ ] Click "Welcome" tab
- [ ] Tab becomes active (highlighted)
- [ ] Status message updates
- [ ] Content changes to Welcome view

- [ ] Click "Organization" tab
- [ ] Tab becomes active
- [ ] Content changes to Organization view

- [ ] Click "Keys" tab
- [ ] Tab becomes active
- [ ] Content changes to Keys view

- [ ] Click "Export" tab
- [ ] Tab becomes active
- [ ] Content changes to Export view

### 5.2 State Persistence
- [ ] Add person in Organization tab
- [ ] Switch to Keys tab
- [ ] Switch back to Organization tab
- [ ] Person is still in graph
- [ ] Form state is preserved

### 5.3 Workflow Navigation
- [ ] Create new domain in Welcome
- [ ] Automatically transitions to Organization
- [ ] Add people
- [ ] Switch to Keys tab
- [ ] Generate all keys
- [ ] Automatically transitions to Export

---

## 6. UI Theme & Styling Tests

### 6.1 Cowboy Theme
- [ ] Background uses dark theme
- [ ] Primary colors are visible (orange/amber)
- [ ] Secondary colors are subtle
- [ ] Text is readable (high contrast)

### 6.2 Glassmorphism Effects
- [ ] Tab bar has glass container style
- [ ] Buttons have glass effect when inactive
- [ ] Active tabs have gradient background
- [ ] Hover effects work on buttons
- [ ] Borders are semi-transparent

### 6.3 Button Styles
- [ ] Primary buttons (orange gradient)
- [ ] Glass buttons (transparent with border)
- [ ] Security buttons (special styling)
- [ ] All buttons have rounded corners
- [ ] Active buttons have glow shadow

### 6.4 Cards & Containers
- [ ] Cards have subtle background
- [ ] Borders are consistent
- [ ] Padding is uniform
- [ ] Shadows add depth

### 6.5 Typography
- [ ] Monospace font for logo
- [ ] Sans-serif for body text
- [ ] Font sizes are hierarchical
- [ ] Line heights are comfortable

---

## 7. Animated Background Tests

### 7.1 Firefly Shader
- [ ] Animated background renders
- [ ] Fireflies are visible
- [ ] Movement appears smooth
- [ ] Animation runs at ~30 FPS
- [ ] No performance issues

### 7.2 Synchronization
- [ ] Fireflies synchronize over time (Kuramoto model)
- [ ] Phase coupling is visible
- [ ] Synchronization doesn't freeze UI
- [ ] CPU usage is reasonable

### 7.3 Performance
- [ ] Animation subscription updates regularly
- [ ] Animation time increments smoothly
- [ ] No frame drops during interaction
- [ ] Shader compiles successfully

---

## 8. Error Handling Tests

### 8.1 Error Display
- [ ] Error message appears in red banner
- [ ] Error icon (❌) is visible
- [ ] Error text is readable
- [ ] Close button (✕) works
- [ ] Error disappears when dismissed

### 8.2 Error Scenarios
- [ ] Invalid file load
- [ ] Failed key generation
- [ ] Missing required fields
- [ ] Permission denied on export
- [ ] All show appropriate errors

### 8.3 Status Messages
- [ ] Status bar shows current operation
- [ ] Status updates in real-time
- [ ] Status persists appropriately
- [ ] Status color indicates state

---

## 9. MVI Architecture Tests

### 9.1 Intent Processing
- [ ] UI emits UiGenerateIntermediateCAClicked
- [ ] Intent is wrapped in MviIntent message
- [ ] Update function is called
- [ ] Model is updated immutably
- [ ] Command is executed
- [ ] Task returns new intent

### 9.2 Model Updates
- [ ] Model is cloned before update
- [ ] Original model is not mutated
- [ ] Updated model replaces old model
- [ ] State changes reflect in UI

### 9.3 Port Dependency Injection
- [ ] Storage port is injected
- [ ] X509 port is injected
- [ ] SSH port is injected
- [ ] YubiKey port is injected
- [ ] Ports are used through commands only

### 9.4 Event Source Separation
- [ ] UI events are distinct from domain events
- [ ] Port events are separate
- [ ] System events are separate
- [ ] Error events are separate

---

## 10. Offline & Air-Gapped Tests

### 10.1 Offline Operation
- [ ] Application runs without network
- [ ] All features work offline
- [ ] No network calls are made
- [ ] Data is local only

### 10.2 File System Storage
- [ ] Events written to JSON files
- [ ] Keys stored in output directory
- [ ] Certificates saved to filesystem
- [ ] Manifest tracks all artifacts

### 10.3 Encrypted Storage Ready
- [ ] Export prepares for SD card
- [ ] Directory structure is correct
- [ ] Manifest includes all resources
- [ ] Ready for encryption layer

---

## 11. Data Integrity Tests

### 11.1 UUID Generation
- [ ] All UUIDs use v7 (time-ordered)
- [ ] No UUID v4 or v5 is used
- [ ] UUIDs are unique
- [ ] UUIDs are sortable by creation time

### 11.2 Event Correlation
- [ ] Events have correlation_id
- [ ] Events have causation_id
- [ ] Event chains are traceable
- [ ] Causation links parent events

### 11.3 Projection Consistency
- [ ] Events applied to projection
- [ ] State matches event history
- [ ] Manifest stays in sync
- [ ] No state loss on error

---

## 12. Performance Tests

### 12.1 Responsiveness
- [ ] UI responds to clicks < 100ms
- [ ] Tab switching is instant
- [ ] Form inputs have no lag
- [ ] Smooth scrolling

### 12.2 Large Data Sets
- [ ] Load 50+ people in organization
- [ ] Graph renders all nodes
- [ ] No slowdown in UI
- [ ] Scrolling remains smooth

### 12.3 Key Generation Scale
- [ ] Generate 100+ keys
- [ ] Progress updates smoothly
- [ ] No UI freeze
- [ ] Memory usage reasonable

### 12.4 Animation Performance
- [ ] Firefly shader at 30 FPS
- [ ] CPU usage < 20% (on modern hardware)
- [ ] No interference with main UI
- [ ] Consistent frame times

---

## 13. Accessibility Tests

### 13.1 Keyboard Navigation
- [ ] Tab key moves between fields
- [ ] Enter key submits forms
- [ ] Escape key dismisses errors
- [ ] All buttons are keyboard accessible

### 13.2 Screen Reader Ready
- [ ] Text elements are readable
- [ ] Buttons have clear labels
- [ ] Status messages are announced
- [ ] Form labels are associated

### 13.3 Color Contrast
- [ ] Text has sufficient contrast
- [ ] Primary text: white on dark
- [ ] Secondary text: light gray
- [ ] Error text: visible red

---

## 14. Edge Cases Tests

### 14.1 Empty States
- [ ] Empty organization graph
- [ ] No people added
- [ ] No keys generated
- [ ] Empty certificate list

### 14.2 Boundary Conditions
- [ ] Very long person names
- [ ] Very long email addresses
- [ ] Many SANs in certificate
- [ ] Long file paths

### 14.3 Special Characters
- [ ] Names with unicode
- [ ] Emails with special chars
- [ ] Domains with hyphens
- [ ] Paths with spaces

---

## 15. Integration Tests

### 15.1 End-to-End Flow
- [ ] Start application
- [ ] Create new domain
- [ ] Add 5 people
- [ ] Generate Root CA
- [ ] Generate 2 Intermediate CAs
- [ ] Generate 3 Server Certificates
- [ ] Generate SSH keys
- [ ] Export domain
- [ ] Verify all files created
- [ ] Check manifest completeness

### 15.2 Load and Continue
- [ ] Create domain and add people
- [ ] Export configuration
- [ ] Close application
- [ ] Restart application
- [ ] Load exported domain
- [ ] Verify all data present
- [ ] Continue key generation
- [ ] Export updated domain

---

## 16. WASM Compatibility Tests (Future)

### 16.1 Build for WASM
- [ ] `./build-wasm.sh` succeeds
- [ ] WASM bundle is created
- [ ] Assets are embedded
- [ ] No native dependencies

### 16.2 Browser Operation
- [ ] Application loads in browser
- [ ] All tabs render
- [ ] Forms work
- [ ] File picker works (browser API)
- [ ] Export downloads file

---

## Test Execution Notes

### Prerequisites
1. Build application: `cargo build --features gui`
2. Create test output directory: `mkdir -p /tmp/cim-keys-test`
3. Prepare test bootstrap config (optional)

### Running Tests
```bash
# Run GUI application
cargo run --features gui -- /tmp/cim-keys-test

# Or use compiled binary
./target/debug/cim-keys-gui /tmp/cim-keys-test
```

### Test Data
Sample bootstrap configuration location: `secrets/domain-bootstrap.json`

### Automation Possibilities
- Manual UI testing (this document)
- Future: Automated integration tests
- Future: Visual regression tests
- Future: Performance benchmarks

---

## Known Issues & TODOs

### Not Yet Implemented
1. SSH key generation (Message handler exists, implementation pending)
2. YubiKey provisioning (GUI trigger exists, hardware integration needed)
3. Remove person functionality (UI event defined, handler incomplete)
4. WASM file loading (Native works, WASM needs implementation)
5. Graph edge creation (UI for relationships between people)
6. Auto-layout algorithm for graph

### Future Enhancements
1. Drag-and-drop nodes in graph
2. Zoom and pan in graph canvas
3. Certificate chain visualization
4. Key usage statistics
5. Audit log viewer
6. Export to multiple formats
7. Import from various sources
8. Real-time collaboration (future: NATS integration)

---

## Testing Checklist Summary

Total Test Cases: 200+

### Critical Path (Must Pass)
- [ ] Application launches
- [ ] Domain creation works
- [ ] People can be added
- [ ] Keys can be generated
- [ ] Domain can be exported

### High Priority
- [ ] All tabs navigate correctly
- [ ] Forms validate inputs
- [ ] Error handling works
- [ ] MVI pattern functions correctly
- [ ] Theme renders properly

### Medium Priority
- [ ] Graph visualization works
- [ ] Animation performs well
- [ ] Multiple CAs can be generated
- [ ] Status messages update

### Low Priority
- [ ] Keyboard navigation
- [ ] Edge cases handled
- [ ] Long text displays correctly
- [ ] Unicode support

---

## Test Results Template

Date: _________
Tester: _________
Version: _________

### Welcome Tab
- [ ] All tests passed
- Issues: ___________

### Organization Tab
- [ ] All tests passed
- Issues: ___________

### Keys Tab
- [ ] All tests passed
- Issues: ___________

### Export Tab
- [ ] All tests passed
- Issues: ___________

### Overall Assessment
- [ ] Ready for production
- [ ] Needs minor fixes
- [ ] Needs major work

---

## Conclusion

This test plan covers all major functionality of the cim-keys GUI application. Execute tests in order, documenting any failures or unexpected behavior. The application follows event-sourced architecture with MVI pattern, so pay special attention to state management and event flow during testing.

**Next Steps:**
1. Execute manual tests following this plan
2. Document results
3. File issues for any bugs found
4. Implement missing functionality (SSH keys, YubiKey)
5. Add automated tests where possible
6. Prepare for WASM deployment
