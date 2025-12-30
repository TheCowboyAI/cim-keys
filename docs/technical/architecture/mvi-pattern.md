# MVI Architecture - Visual Overview

## Complete System Architecture

```mermaid
graph TB
    subgraph "User Interaction Layer"
        User[👤 User] --> UI[🖥️ UI Elements<br/>Buttons, Inputs, Forms]
    end

    subgraph "MVI Layer - Pure Functional Reactive"
        UI --> Intent[📨 Intent Enum<br/>Unified Event Source]

        Intent --> Update[⚙️ update Function<br/>Pure State Transition]
        Model[📦 Model<br/>Pure Immutable State] --> Update

        Update --> NewModel[📦 Updated Model]
        Update --> Command[⚡ Command<br/>Effect Description]

        NewModel --> View[🎨 view Function<br/>Pure Rendering]
        View --> UI
    end

    subgraph "Hexagonal Ports - Interfaces"
        Command --> Storage[💾 StoragePort]
        Command --> X509[📜 X509Port]
        Command --> SSH[🔑 SshKeyPort]
        Command --> YubiKey[🔐 YubiKeyPort]
    end

    subgraph "Adapters - Implementations"
        Storage --> InMemory[InMemoryAdapter]
        Storage --> FileSystem[FileSystemAdapter]

        X509 --> MockX509[MockX509Adapter]
        X509 --> Rcgen[RcgenAdapter]

        SSH --> MockSSH[MockSshAdapter]
        SSH --> SshKeys[SshKeysAdapter]

        YubiKey --> MockYubiKey[MockYubiKeyAdapter]
        YubiKey --> PCSC[YubiKeyPCSCAdapter]
    end

    subgraph "Event Flow Back"
        InMemory --> Intent
        FileSystem --> Intent
        MockX509 --> Intent
        Rcgen --> Intent
        MockSSH --> Intent
        SshKeys --> Intent
        MockYubiKey --> Intent
        PCSC --> Intent
    end

    style Intent fill:#e1f5ff,stroke:#0066cc,stroke-width:3px
    style Update fill:#fff3cd,stroke:#ff9900,stroke-width:3px
    style Model fill:#d4edda,stroke:#28a745,stroke-width:3px
    style NewModel fill:#d4edda,stroke:#28a745,stroke-width:3px
    style Command fill:#f8d7da,stroke:#dc3545,stroke-width:3px
    style View fill:#d1ecf1,stroke:#17a2b8,stroke-width:3px
```

## Intent Flow Diagram

```mermaid
sequenceDiagram
    participant User
    participant View
    participant Intent
    participant Update
    participant Model
    participant Command
    participant Port
    participant Adapter

    User->>View: Click "Generate Root CA"
    View->>Intent: Intent::UiGenerateRootCAClicked
    Intent->>Update: Process intent
    Update->>Model: model.with_status("Generating...")
    Model-->>Update: Updated Model
    Update->>Command: Task::perform(async { ... })
    Update-->>View: (Updated Model, Command)

    View->>User: Display "Generating..."

    Command->>Port: x509.generate_root_ca().await
    Port->>Adapter: Execute generation
    Adapter-->>Port: Certificate
    Port-->>Command: Ok(cert)

    Command->>Intent: Intent::PortX509RootCAGenerated { cert }
    Intent->>Update: Process port response
    Update->>Model: model.with_root_ca_generated()
    Model-->>Update: Updated Model
    Update-->>View: (Updated Model, Task::none())

    View->>User: Display "✓ Generated"
```

## Event Source Categorization

```mermaid
graph LR
    subgraph "Event Sources"
        UI[UI Events<br/>Button clicks<br/>Text input<br/>Tab selection]
        Ports[Port Events<br/>Async responses<br/>Storage ops<br/>Crypto ops]
        Domain[Domain Events<br/>Aggregate events<br/>Business logic<br/>State changes]
        System[System Events<br/>File picker<br/>Clipboard<br/>Errors]
    end

    subgraph "Intent Variants"
        UI --> UiIntent[Ui*<br/>UiGenerateRootCAClicked<br/>UiTabSelected<br/>UiAddPersonClicked]
        Ports --> PortIntent[Port*<br/>PortX509RootCAGenerated<br/>PortSSHKeypairGenerated<br/>PortStorageWriteCompleted]
        Domain --> DomainIntent[Domain*<br/>DomainCreated<br/>PersonAdded<br/>RootCAGenerated]
        System --> SystemIntent[System*<br/>SystemFileSelected<br/>SystemErrorOccurred<br/>SystemClipboardUpdated]
    end

    subgraph "Single Intent Enum"
        UiIntent --> Intent[Intent Enum<br/>ALL Events]
        PortIntent --> Intent
        DomainIntent --> Intent
        SystemIntent --> Intent
    end

    style Intent fill:#e1f5ff,stroke:#0066cc,stroke-width:4px
```

## Data Flow: Root CA Generation Workflow

```mermaid
graph TD
    Start[User Clicks<br/>"Generate Root CA"] --> Intent1[Intent::UiGenerateRootCAClicked]

    Intent1 --> Update1[update Function]
    Update1 --> Model1[Model<br/>status: "Generating..."<br/>progress: 0.1]
    Update1 --> Cmd1[Command:<br/>Task::perform<br/>x509.generate_root_ca]

    Model1 --> View1[view Function]
    View1 --> Display1[Display:<br/>"Generating Root CA..."<br/>Progress: 10%]

    Cmd1 --> Exec[Execute Async]
    Exec --> Port1[X509Port]
    Port1 --> Adapter1[MockX509Adapter<br/>or<br/>RcgenAdapter]

    Adapter1 --> Result{Success?}

    Result -->|Yes| Intent2[Intent::PortX509RootCAGenerated<br/>certificate_pem<br/>private_key_pem]
    Result -->|No| Intent3[Intent::PortX509GenerationFailed<br/>error]

    Intent2 --> Update2[update Function]
    Intent3 --> Update3[update Function]

    Update2 --> Model2[Model<br/>root_ca_generated: true<br/>status: "✓ Generated"<br/>progress: 0.5]
    Update3 --> Model3[Model<br/>error: Some(error)<br/>progress: 0.0]

    Model2 --> View2[view Function]
    Model3 --> View3[view Function]

    View2 --> Display2[Display:<br/>"✓ Root CA Generated"<br/>Progress: 50%]
    View3 --> Display3[Display:<br/>"✗ Generation Failed"<br/>Show Error]

    style Intent1 fill:#e1f5ff
    style Intent2 fill:#e1f5ff
    style Intent3 fill:#e1f5ff
    style Update1 fill:#fff3cd
    style Update2 fill:#fff3cd
    style Update3 fill:#fff3cd
    style Model1 fill:#d4edda
    style Model2 fill:#d4edda
    style Model3 fill:#d4edda
    style Cmd1 fill:#f8d7da
```

## Model State Transitions

```mermaid
stateDiagram-v2
    [*] --> Welcome: App Start

    Welcome --> Creating: UiCreateDomainClicked
    Creating --> Created: DomainCreated
    Creating --> LoadError: Error

    Created --> Organization: UiTabSelected(Organization)
    Organization --> Keys: UiTabSelected(Keys)
    Keys --> Export: UiTabSelected(Export)

    Keys --> GeneratingRootCA: UiGenerateRootCAClicked
    GeneratingRootCA --> RootCAComplete: PortX509RootCAGenerated
    GeneratingRootCA --> Keys: PortX509GenerationFailed

    Keys --> GeneratingSSH: UiGenerateSSHKeysClicked
    GeneratingSSH --> SSHComplete: PortSSHKeypairGenerated
    GeneratingSSH --> Keys: PortSSHGenerationFailed

    Keys --> ProvisioningYubiKey: UiProvisionYubiKeyClicked
    ProvisioningYubiKey --> YubiKeyComplete: PortYubiKeyKeyGenerated
    ProvisioningYubiKey --> Keys: PortYubiKeyOperationFailed

    Export --> Exporting: UiExportClicked
    Exporting --> ExportComplete: PortStorageWriteCompleted
    Exporting --> ExportFailed: PortStorageWriteFailed

    note right of Created
        Domain Ready:
        - Organization created
        - People added
        - Ready for key generation
    end note

    note right of RootCAComplete
        Root CA Generated:
        - Certificate created
        - Private key secured
        - Ready for signing
    end note
```

## Component Interaction Matrix

| Component | Intent | Model | Update | View | Ports | Adapters |
|-----------|--------|-------|--------|------|-------|----------|
| **Intent** | - | ✓ Read | ✓ Input | ✓ Emit | ✓ Return | ✗ |
| **Model** | ✗ | - | ✓ Clone | ✓ Read | ✗ | ✗ |
| **Update** | ✓ Match | ✓ Transform | - | ✗ | ✓ Call | ✗ |
| **View** | ✓ Create | ✓ Read | ✗ | - | ✗ | ✗ |
| **Ports** | ✓ Return | ✗ | ✓ Injected | ✗ | - | ✓ Call |
| **Adapters** | ✓ Create | ✗ | ✗ | ✗ | ✓ Impl | - |

**Legend**:
- ✓ = Direct interaction
- ✗ = No direct interaction
- Read = Read-only access
- Clone = Creates copy
- Match = Pattern matching
- Transform = State transformation
- Create = Creates new instances
- Emit = Generates events
- Input = Receives as parameter
- Return = Returns as result
- Call = Makes function/method calls
- Injected = Dependency injected
- Impl = Implements interface

## Dependency Graph

```mermaid
graph TB
    subgraph "Pure Layer - No Dependencies"
        Intent[Intent Enum]
        Model[Model Struct]
    end

    subgraph "Pure Logic Layer"
        Update[update Function] --> Intent
        Update --> Model
        View[view Function] --> Model
        View --> Intent
    end

    subgraph "Port Interfaces - No Implementation"
        StoragePort[StoragePort Trait]
        X509Port[X509Port Trait]
        SshPort[SshKeyPort Trait]
        YubiKeyPort[YubiKeyPort Trait]
    end

    subgraph "Adapter Implementations"
        InMemory[InMemoryAdapter] -.implements.-> StoragePort
        FileSystem[FileSystemAdapter] -.implements.-> StoragePort
        MockX509[MockX509Adapter] -.implements.-> X509Port
        Rcgen[RcgenAdapter] -.implements.-> X509Port
        MockSSH[MockSshAdapter] -.implements.-> SshPort
        SshKeys[SshKeysAdapter] -.implements.-> SshPort
        MockYubiKey[MockYubiKeyAdapter] -.implements.-> YubiKeyPort
        PCSC[YubiKeyPCSCAdapter] -.implements.-> YubiKeyPort
    end

    Update -.uses via Arc.-> StoragePort
    Update -.uses via Arc.-> X509Port
    Update -.uses via Arc.-> SshPort
    Update -.uses via Arc.-> YubiKeyPort

    style Intent fill:#e1f5ff
    style Model fill:#d4edda
    style Update fill:#fff3cd
    style View fill:#d1ecf1
```

## File Structure

```
cim-keys/
├── src/
│   ├── mvi/                          ← NEW: MVI Architecture
│   │   ├── mod.rs                    (19 lines)
│   │   ├── intent.rs                 (261 lines) ← Event Sources
│   │   ├── model.rs                  (217 lines) ← Pure State
│   │   ├── update.rs                 (449 lines) ← State Transitions
│   │   └── view.rs                   (446 lines) ← Pure Rendering
│   │
│   ├── ports/                        ← Hexagonal Ports
│   │   ├── storage.rs
│   │   ├── yubikey.rs
│   │   ├── x509.rs
│   │   ├── gpg.rs
│   │   └── ssh.rs
│   │
│   ├── adapters/                     ← Port Implementations
│   │   ├── in_memory.rs
│   │   ├── yubikey_mock.rs
│   │   ├── x509_mock.rs
│   │   ├── gpg_mock.rs
│   │   └── ssh_mock.rs
│   │
│   └── gui.rs                        ← Old GUI (to be migrated)
│
├── examples/
│   ├── mvi_demo.rs                   ← NEW: Working MVI Example
│   └── hexagonal_demo.rs             ← Existing: Port Demo
│
└── docs/                             ← NEW: MVI Documentation
    ├── MVI_IMPLEMENTATION_GUIDE.md
    ├── MVI_IMPLEMENTATION_SUMMARY.md
    └── MVI_COMPLETION_REPORT.md
```

## Summary

The MVI architecture provides:

1. **Clear Event Flow** - All events categorized by origin
2. **Pure Functions** - Update and View are completely pure
3. **Hexagonal Integration** - Ports dependency-injected
4. **Type Safety** - Compiler enforces correct patterns
5. **Cross-Framework** - Core logic framework-independent

**Total Implementation**: ~2,000 lines of production-ready code
**Status**: ✅ Complete and ready for integration
