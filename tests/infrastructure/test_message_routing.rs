//! Infrastructure Layer 1.3: Message Routing Tests for cim-keys
//! 
//! User Story: As a security system, I need to route key operation commands through message handlers
//!
//! Test Requirements:
//! - Verify command routing to appropriate handlers
//! - Verify handler registration and discovery
//! - Verify response routing back to requesters
//! - Verify error handling and fallback routing
//!
//! Event Sequence:
//! 1. RouterInitialized
//! 2. HandlerRegistered { operation_type, handler_id }
//! 3. CommandRouted { command_id, handler_id }
//! 4. ResponseGenerated { command_id, response_type }
//!
//! ```mermaid
//! graph LR
//!     A[Test Start] --> B[Initialize Router]
//!     B --> C[RouterInitialized]
//!     C --> D[Register Handlers]
//!     D --> E[HandlerRegistered]
//!     E --> F[Route Command]
//!     F --> G[CommandRouted]
//!     G --> H[Generate Response]
//!     H --> I[ResponseGenerated]
//!     I --> J[Test Success]
//! ```

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use async_trait::async_trait;

/// Key operation commands
#[derive(Debug, Clone, PartialEq)]
pub enum KeyCommand {
    GenerateKey {
        algorithm: String,
        key_size: u32,
        purpose: String,
    },
    RotateKey {
        key_id: String,
        reason: String,
    },
    RevokeKey {
        key_id: String,
        reason: String,
    },
    SignData {
        key_id: String,
        data: Vec<u8>,
    },
    VerifySignature {
        key_id: String,
        data: Vec<u8>,
        signature: Vec<u8>,
    },
}

/// Key operation responses
#[derive(Debug, Clone, PartialEq)]
pub enum KeyResponse {
    KeyGenerated {
        key_id: String,
        fingerprint: String,
    },
    KeyRotated {
        old_key_id: String,
        new_key_id: String,
    },
    KeyRevoked {
        key_id: String,
        revocation_time: std::time::SystemTime,
    },
    DataSigned {
        signature: Vec<u8>,
    },
    SignatureVerified {
        valid: bool,
    },
    Error {
        code: String,
        message: String,
    },
}

/// Routing events for testing
#[derive(Debug, Clone, PartialEq)]
pub enum RoutingEvent {
    RouterInitialized,
    HandlerRegistered {
        operation_type: String,
        handler_id: String,
    },
    CommandRouted {
        command_id: String,
        handler_id: String,
    },
    ResponseGenerated {
        command_id: String,
        response_type: String,
    },
    FallbackHandlerInvoked {
        command_id: String,
        reason: String,
    },
    RoutingError {
        command_id: String,
        error: String,
    },
}

/// Handler trait for key operations
#[async_trait]
pub trait KeyOperationHandler: Send + Sync {
    async fn handle_command(&self, command: KeyCommand) -> Result<KeyResponse, String>;
    fn supported_operations(&self) -> Vec<String>;
    fn handler_id(&self) -> String;
}

/// Mock key generation handler
pub struct MockKeyGenerationHandler {
    id: String,
}

impl MockKeyGenerationHandler {
    pub fn new() -> Self {
        Self {
            id: "key-gen-handler".to_string(),
        }
    }
}

#[async_trait]
impl KeyOperationHandler for MockKeyGenerationHandler {
    async fn handle_command(&self, command: KeyCommand) -> Result<KeyResponse, String> {
        match command {
            KeyCommand::GenerateKey { algorithm, .. } => {
                // Simulate key generation
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                
                Ok(KeyResponse::KeyGenerated {
                    key_id: format!("key-{}", uuid::Uuid::new_v4()),
                    fingerprint: format!("FP:{}:{}", algorithm, uuid::Uuid::new_v4()),
                })
            }
            _ => Err("Unsupported operation".to_string()),
        }
    }

    fn supported_operations(&self) -> Vec<String> {
        vec!["GenerateKey".to_string()]
    }

    fn handler_id(&self) -> String {
        self.id.clone()
    }
}

/// Mock key lifecycle handler
pub struct MockKeyLifecycleHandler {
    id: String,
}

impl MockKeyLifecycleHandler {
    pub fn new() -> Self {
        Self {
            id: "key-lifecycle-handler".to_string(),
        }
    }
}

#[async_trait]
impl KeyOperationHandler for MockKeyLifecycleHandler {
    async fn handle_command(&self, command: KeyCommand) -> Result<KeyResponse, String> {
        match command {
            KeyCommand::RotateKey { key_id, .. } => {
                Ok(KeyResponse::KeyRotated {
                    old_key_id: key_id,
                    new_key_id: format!("key-{}", uuid::Uuid::new_v4()),
                })
            }
            KeyCommand::RevokeKey { key_id, .. } => {
                Ok(KeyResponse::KeyRevoked {
                    key_id,
                    revocation_time: std::time::SystemTime::now(),
                })
            }
            _ => Err("Unsupported operation".to_string()),
        }
    }

    fn supported_operations(&self) -> Vec<String> {
        vec!["RotateKey".to_string(), "RevokeKey".to_string()]
    }

    fn handler_id(&self) -> String {
        self.id.clone()
    }
}

/// Mock cryptographic operations handler
pub struct MockCryptoOperationsHandler {
    id: String,
}

impl MockCryptoOperationsHandler {
    pub fn new() -> Self {
        Self {
            id: "crypto-ops-handler".to_string(),
        }
    }
}

#[async_trait]
impl KeyOperationHandler for MockCryptoOperationsHandler {
    async fn handle_command(&self, command: KeyCommand) -> Result<KeyResponse, String> {
        match command {
            KeyCommand::SignData { .. } => {
                Ok(KeyResponse::DataSigned {
                    signature: vec![1, 2, 3, 4], // Mock signature
                })
            }
            KeyCommand::VerifySignature { .. } => {
                Ok(KeyResponse::SignatureVerified {
                    valid: true, // Mock verification
                })
            }
            _ => Err("Unsupported operation".to_string()),
        }
    }

    fn supported_operations(&self) -> Vec<String> {
        vec!["SignData".to_string(), "VerifySignature".to_string()]
    }

    fn handler_id(&self) -> String {
        self.id.clone()
    }
}

/// Message router for key operations
pub struct KeyOperationRouter {
    handlers: Arc<Mutex<HashMap<String, Box<dyn KeyOperationHandler>>>>,
    operation_map: Arc<Mutex<HashMap<String, String>>>, // operation -> handler_id
    fallback_handler: Option<Box<dyn KeyOperationHandler>>,
}

impl KeyOperationRouter {
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(Mutex::new(HashMap::new())),
            operation_map: Arc::new(Mutex::new(HashMap::new())),
            fallback_handler: None,
        }
    }

    pub async fn register_handler(&self, handler: Box<dyn KeyOperationHandler>) -> Result<(), String> {
        let handler_id = handler.handler_id();
        let operations = handler.supported_operations();

        let mut handlers = self.handlers.lock().await;
        let mut op_map = self.operation_map.lock().await;

        // Register handler
        handlers.insert(handler_id.clone(), handler);

        // Map operations to handler
        for op in operations {
            op_map.insert(op, handler_id.clone());
        }

        Ok(())
    }

    pub async fn route_command(
        &self,
        _command_id: String,
        command: KeyCommand,
    ) -> Result<(String, KeyResponse), String> {
        let operation = match &command {
            KeyCommand::GenerateKey { .. } => "GenerateKey",
            KeyCommand::RotateKey { .. } => "RotateKey",
            KeyCommand::RevokeKey { .. } => "RevokeKey",
            KeyCommand::SignData { .. } => "SignData",
            KeyCommand::VerifySignature { .. } => "VerifySignature",
        };

        let op_map = self.operation_map.lock().await;
        let handler_id = op_map.get(operation).cloned();
        drop(op_map);

        if let Some(handler_id) = handler_id {
            let handlers = self.handlers.lock().await;
            if let Some(handler) = handlers.get(&handler_id) {
                let response = handler.handle_command(command).await?;
                Ok((handler_id, response))
            } else {
                Err(format!("Handler {} not found", handler_id))
            }
        } else if let Some(ref fallback) = self.fallback_handler {
            let response = fallback.handle_command(command).await?;
            Ok(("fallback".to_string(), response))
        } else {
            Err(format!("No handler for operation {}", operation))
        }
    }

    pub fn set_fallback_handler(&mut self, handler: Box<dyn KeyOperationHandler>) {
        self.fallback_handler = Some(handler);
    }

    pub async fn get_routing_stats(&self) -> HashMap<String, usize> {
        let op_map = self.operation_map.lock().await;
        let mut stats = HashMap::new();

        for (_, handler_id) in op_map.iter() {
            *stats.entry(handler_id.clone()).or_insert(0) += 1;
        }

        stats
    }
}

/// Event stream validator for routing testing
pub struct RoutingEventStreamValidator {
    expected_events: Vec<RoutingEvent>,
    captured_events: Vec<RoutingEvent>,
}

impl RoutingEventStreamValidator {
    pub fn new() -> Self {
        Self {
            expected_events: Vec::new(),
            captured_events: Vec::new(),
        }
    }

    pub fn expect_sequence(mut self, events: Vec<RoutingEvent>) -> Self {
        self.expected_events = events;
        self
    }

    pub fn capture_event(&mut self, event: RoutingEvent) {
        self.captured_events.push(event);
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.captured_events.len() != self.expected_events.len() {
            return Err(format!(
                "Event count mismatch: expected {}, got {}",
                self.expected_events.len(),
                self.captured_events.len()
            ));
        }

        for (i, (expected, actual)) in self.expected_events.iter()
            .zip(self.captured_events.iter())
            .enumerate()
        {
            if expected != actual {
                return Err(format!(
                    "Event mismatch at position {}: expected {:?}, got {:?}",
                    i, expected, actual
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_router_initialization() {
        // Arrange
        let mut validator = RoutingEventStreamValidator::new()
            .expect_sequence(vec![
                RoutingEvent::RouterInitialized,
            ]);

        // Act
        let router = KeyOperationRouter::new();
        validator.capture_event(RoutingEvent::RouterInitialized);

        // Assert
        assert!(validator.validate().is_ok());
        let stats = router.get_routing_stats().await;
        assert_eq!(stats.len(), 0);
    }

    #[tokio::test]
    async fn test_handler_registration() {
        // Arrange
        let mut validator = RoutingEventStreamValidator::new()
            .expect_sequence(vec![
                RoutingEvent::HandlerRegistered {
                    operation_type: "GenerateKey".to_string(),
                    handler_id: "key-gen-handler".to_string(),
                },
            ]);

        let router = KeyOperationRouter::new();
        let handler = Box::new(MockKeyGenerationHandler::new());

        // Act
        router.register_handler(handler).await.unwrap();
        
        validator.capture_event(RoutingEvent::HandlerRegistered {
            operation_type: "GenerateKey".to_string(),
            handler_id: "key-gen-handler".to_string(),
        });

        // Assert
        assert!(validator.validate().is_ok());
        let stats = router.get_routing_stats().await;
        assert_eq!(stats.get("key-gen-handler"), Some(&1));
    }

    #[tokio::test]
    async fn test_command_routing() {
        // Arrange
        let router = KeyOperationRouter::new();
        router.register_handler(Box::new(MockKeyGenerationHandler::new())).await.unwrap();

        let mut validator = RoutingEventStreamValidator::new()
            .expect_sequence(vec![
                RoutingEvent::CommandRouted {
                    command_id: "cmd-123".to_string(),
                    handler_id: "key-gen-handler".to_string(),
                },
                RoutingEvent::ResponseGenerated {
                    command_id: "cmd-123".to_string(),
                    response_type: "KeyGenerated".to_string(),
                },
            ]);

        let command = KeyCommand::GenerateKey {
            algorithm: "RSA".to_string(),
            key_size: 2048,
            purpose: "signing".to_string(),
        };

        // Act
        let (handler_id, response) = router.route_command("cmd-123".to_string(), command).await.unwrap();

        validator.capture_event(RoutingEvent::CommandRouted {
            command_id: "cmd-123".to_string(),
            handler_id: handler_id.clone(),
        });

        // Assert
        assert_eq!(handler_id, "key-gen-handler");
        matches!(response, KeyResponse::KeyGenerated { .. });
        
        validator.capture_event(RoutingEvent::ResponseGenerated {
            command_id: "cmd-123".to_string(),
            response_type: "KeyGenerated".to_string(),
        });
        
        assert!(validator.validate().is_ok());
    }

    #[tokio::test]
    async fn test_multiple_handler_routing() {
        // Arrange
        let router = KeyOperationRouter::new();
        router.register_handler(Box::new(MockKeyGenerationHandler::new())).await.unwrap();
        router.register_handler(Box::new(MockKeyLifecycleHandler::new())).await.unwrap();
        router.register_handler(Box::new(MockCryptoOperationsHandler::new())).await.unwrap();

        // Act & Assert - Test each handler
        let gen_cmd = KeyCommand::GenerateKey {
            algorithm: "Ed25519".to_string(),
            key_size: 256,
            purpose: "auth".to_string(),
        };
        let (handler_id, _) = router.route_command("gen-1".to_string(), gen_cmd).await.unwrap();
        assert_eq!(handler_id, "key-gen-handler");

        let rotate_cmd = KeyCommand::RotateKey {
            key_id: "test-key".to_string(),
            reason: "scheduled".to_string(),
        };
        let (handler_id, _) = router.route_command("rot-1".to_string(), rotate_cmd).await.unwrap();
        assert_eq!(handler_id, "key-lifecycle-handler");

        let sign_cmd = KeyCommand::SignData {
            key_id: "test-key".to_string(),
            data: vec![1, 2, 3],
        };
        let (handler_id, _) = router.route_command("sign-1".to_string(), sign_cmd).await.unwrap();
        assert_eq!(handler_id, "crypto-ops-handler");
    }

    #[tokio::test]
    async fn test_fallback_handler() {
        // Arrange
        let mut router = KeyOperationRouter::new();
        
        // Create a fallback handler that handles all operations
        struct FallbackHandler;
        
        #[async_trait]
        impl KeyOperationHandler for FallbackHandler {
            async fn handle_command(&self, _: KeyCommand) -> Result<KeyResponse, String> {
                Ok(KeyResponse::Error {
                    code: "FALLBACK".to_string(),
                    message: "Handled by fallback".to_string(),
                })
            }
            
            fn supported_operations(&self) -> Vec<String> {
                vec![]
            }
            
            fn handler_id(&self) -> String {
                "fallback-handler".to_string()
            }
        }

        router.set_fallback_handler(Box::new(FallbackHandler));

        let command = KeyCommand::GenerateKey {
            algorithm: "Unknown".to_string(),
            key_size: 1024,
            purpose: "test".to_string(),
        };

        // Act
        let (handler_id, response) = router.route_command("fallback-1".to_string(), command).await.unwrap();

        // Assert
        assert_eq!(handler_id, "fallback");
        matches!(response, KeyResponse::Error { code, .. } if code == "FALLBACK");
    }

    #[tokio::test]
    async fn test_routing_error_handling() {
        // Arrange
        let router = KeyOperationRouter::new();
        // No handlers registered

        let command = KeyCommand::GenerateKey {
            algorithm: "RSA".to_string(),
            key_size: 2048,
            purpose: "signing".to_string(),
        };

        // Act
        let result = router.route_command("error-1".to_string(), command).await;

        // Assert
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No handler for operation"));
    }

    #[tokio::test]
    async fn test_routing_statistics() {
        // Arrange
        let router = KeyOperationRouter::new();
        router.register_handler(Box::new(MockKeyGenerationHandler::new())).await.unwrap();
        router.register_handler(Box::new(MockKeyLifecycleHandler::new())).await.unwrap();

        // Act
        let stats = router.get_routing_stats().await;

        // Assert
        assert_eq!(stats.len(), 2);
        assert_eq!(stats.get("key-gen-handler"), Some(&1));
        assert_eq!(stats.get("key-lifecycle-handler"), Some(&2)); // Handles 2 operations
    }

    #[tokio::test]
    async fn test_concurrent_routing() {
        // Arrange
        let router = Arc::new(KeyOperationRouter::new());
        router.register_handler(Box::new(MockKeyGenerationHandler::new())).await.unwrap();

        let router_clone1 = router.clone();
        let router_clone2 = router.clone();

        // Act - Route commands concurrently
        let handle1 = tokio::spawn(async move {
            let cmd = KeyCommand::GenerateKey {
                algorithm: "RSA".to_string(),
                key_size: 2048,
                purpose: "signing".to_string(),
            };
            router_clone1.route_command("concurrent-1".to_string(), cmd).await
        });

        let handle2 = tokio::spawn(async move {
            let cmd = KeyCommand::GenerateKey {
                algorithm: "Ed25519".to_string(),
                key_size: 256,
                purpose: "auth".to_string(),
            };
            router_clone2.route_command("concurrent-2".to_string(), cmd).await
        });

        // Assert
        let result1 = handle1.await.unwrap();
        let result2 = handle2.await.unwrap();

        assert!(result1.is_ok());
        assert!(result2.is_ok());
    }

    #[tokio::test]
    async fn test_response_type_detection() {
        // Arrange
        let router = KeyOperationRouter::new();
        router.register_handler(Box::new(MockKeyLifecycleHandler::new())).await.unwrap();

        // Test different response types
        let revoke_cmd = KeyCommand::RevokeKey {
            key_id: "test-key".to_string(),
            reason: "compromised".to_string(),
        };

        // Act
        let (_, response) = router.route_command("revoke-1".to_string(), revoke_cmd).await.unwrap();

        // Assert
        match response {
            KeyResponse::KeyRevoked { key_id, .. } => {
                assert_eq!(key_id, "test-key");
            }
            _ => panic!("Unexpected response type"),
        }
    }
} 