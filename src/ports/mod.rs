//! Ports (interfaces) for external system integration
//!
//! This module defines the interfaces that our domain uses to interact
//! with external systems. The domain only knows about these interfaces,
//! not the concrete implementations.
//!
//! **Category Theory Perspective:**
//! Each port is a **Functor** interface that maps from an external category
//! to the Domain category, preserving structure and composition laws.

pub mod nats;
pub mod storage;
pub mod yubikey;
pub mod x509;
pub mod gpg;
pub mod ssh;
pub mod neo4j;

pub use nats::{
    // Key management port
    NatsKeyPort, NatsKeyOperations,
    // JetStream port
    JetStreamPort, JetStreamSubscription, JetStreamMessage, JetStreamHeaders,
    JetStreamError, PublishAck, StreamInfo, ConsumerInfo,
    JetStreamStreamConfig, JetStreamConsumerConfig,
    JsRetentionPolicy, JsStorageType, JsAckPolicy, JsDeliverPolicy,
    // KV store
    KvBucketConfig,
};
pub use storage::{StoragePort, StorageConfig, StorageMetadata, StorageError, SyncMode};
pub use yubikey::{
    YubiKeyPort, YubiKeyDevice, YubiKeyError, PivSlot, KeyAlgorithm,
    PublicKey, Signature, SecureString,
};
pub use x509::{
    X509Port, Certificate, CertificateSubject, CertificateSigningRequest,
    PrivateKey, KeyUsage, ExtendedKeyUsage, CertificateFormat,
    RevokedCertificate, RevocationReason, CertificateRevocationList,
    OcspStatus, OcspResponse, X509Error,
};
pub use gpg::{
    GpgPort, GpgKeyId, GpgKeypair, GpgKeyType, GpgKeyInfo,
    GpgVerification, RevocationReason as GpgRevocationReason, GpgError,
};
pub use ssh::{
    SshKeyPort, SshKeyType, SshKeypair, SshPublicKey, SshPrivateKey,
    SshSignature, SshPrivateKeyFormat, SshPublicKeyFormat,
    FingerprintHashType, KeyConversionFormat, SshError,
};
pub use neo4j::{
    // Neo4j Port trait
    Neo4jPort, Neo4jTransaction,
    // Query types
    CypherQuery, CypherValue, CypherBatch, BatchMetadata,
    // Result types
    ExecutionResult, QueryResult, Record, DatabaseInfo,
    // Graph data types
    GraphNode, GraphEdge, DomainGraphData,
    // Traits for projection
    ToGraphNode, ToGraphEdge,
    // Configuration and errors
    Neo4jConfig, Neo4jError,
};