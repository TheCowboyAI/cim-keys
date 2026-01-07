// Copyright (c) 2025 - Cowboy AI, LLC.

//! Validation Error Types
//!
//! This module defines error types for ACL validation with error accumulation.
//!
//! ## NonEmptyVec Pattern
//!
//! When validation fails, we want ALL errors, not just the first one.
//! `NonEmptyVec<ValidationError>` guarantees at least one error is present
//! and allows collecting all validation failures.
//!
//! ## Example
//!
//! ```rust,ignore
//! use crate::acl::error::{NonEmptyVec, ValidationError};
//!
//! // Accumulate errors during validation
//! let mut errors = Vec::new();
//!
//! if name.is_empty() {
//!     errors.push(ValidationError::new("name", "Name is required"));
//! }
//!
//! if !email.contains('@') {
//!     errors.push(ValidationError::new("email", "Invalid email format"));
//! }
//!
//! // Convert to NonEmptyVec if there are errors
//! if let Some(non_empty) = NonEmptyVec::from_vec(errors) {
//!     return Err(non_empty);
//! }
//! ```

use std::fmt;

/// A validation error for a specific field
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    /// The field that failed validation
    pub field: String,
    /// The error message
    pub message: String,
    /// Optional error code for programmatic handling
    pub code: Option<String>,
}

impl ValidationError {
    /// Create a new validation error
    pub fn new(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            message: message.into(),
            code: None,
        }
    }

    /// Create a new validation error with an error code
    pub fn with_code(
        field: impl Into<String>,
        message: impl Into<String>,
        code: impl Into<String>,
    ) -> Self {
        Self {
            field: field.into(),
            message: message.into(),
            code: Some(code.into()),
        }
    }

    /// Create an error for a required field that is empty
    pub fn required(field: impl Into<String>) -> Self {
        let f = field.into();
        Self {
            field: f.clone(),
            message: format!("{} is required", f),
            code: Some("REQUIRED".to_string()),
        }
    }

    /// Create an error for an invalid email format
    pub fn invalid_email(field: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            message: "Invalid email format".to_string(),
            code: Some("INVALID_EMAIL".to_string()),
        }
    }

    /// Create an error for mismatched confirmation fields
    pub fn mismatch(field: impl Into<String>, confirm_field: &str) -> Self {
        let f = field.into();
        Self {
            field: f.clone(),
            message: format!("{} does not match {}", f, confirm_field),
            code: Some("MISMATCH".to_string()),
        }
    }

    /// Create an error for a value that is too short
    pub fn too_short(field: impl Into<String>, min_length: usize) -> Self {
        let f = field.into();
        Self {
            field: f.clone(),
            message: format!("{} must be at least {} characters", f, min_length),
            code: Some("TOO_SHORT".to_string()),
        }
    }

    /// Create an error for an invalid domain format
    pub fn invalid_domain(field: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            message: "Invalid domain format".to_string(),
            code: Some("INVALID_DOMAIN".to_string()),
        }
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref code) = self.code {
            write!(f, "[{}] {}: {}", code, self.field, self.message)
        } else {
            write!(f, "{}: {}", self.field, self.message)
        }
    }
}

impl std::error::Error for ValidationError {}

/// A non-empty vector guaranteeing at least one element
///
/// This is used to represent validation errors where we know at least
/// one error must exist. It provides the same iteration interface as Vec.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NonEmptyVec<T> {
    head: T,
    tail: Vec<T>,
}

impl<T> NonEmptyVec<T> {
    /// Create a NonEmptyVec with a single element
    pub fn new(head: T) -> Self {
        Self {
            head,
            tail: Vec::new(),
        }
    }

    /// Create a NonEmptyVec from a head and tail
    pub fn from_parts(head: T, tail: Vec<T>) -> Self {
        Self { head, tail }
    }

    /// Try to create a NonEmptyVec from a Vec
    /// Returns None if the Vec is empty
    pub fn from_vec(mut vec: Vec<T>) -> Option<Self> {
        if vec.is_empty() {
            None
        } else {
            let head = vec.remove(0);
            Some(Self { head, tail: vec })
        }
    }

    /// Get the first element
    pub fn head(&self) -> &T {
        &self.head
    }

    /// Get the tail elements
    pub fn tail(&self) -> &[T] {
        &self.tail
    }

    /// Get the length (always >= 1)
    pub fn len(&self) -> usize {
        1 + self.tail.len()
    }

    /// Check if this is a singleton (len == 1)
    pub fn is_singleton(&self) -> bool {
        self.tail.is_empty()
    }

    /// Push an element to the tail
    pub fn push(&mut self, value: T) {
        self.tail.push(value);
    }

    /// Get an iterator over all elements
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        std::iter::once(&self.head).chain(self.tail.iter())
    }

    /// Convert to a Vec
    pub fn into_vec(self) -> Vec<T> {
        let mut vec = Vec::with_capacity(1 + self.tail.len());
        vec.push(self.head);
        vec.extend(self.tail);
        vec
    }

    /// Map a function over all elements
    pub fn map<U, F>(self, mut f: F) -> NonEmptyVec<U>
    where
        F: FnMut(T) -> U,
    {
        NonEmptyVec {
            head: f(self.head),
            tail: self.tail.into_iter().map(f).collect(),
        }
    }

    /// Append another NonEmptyVec to this one
    pub fn append(mut self, other: NonEmptyVec<T>) -> Self {
        self.tail.push(other.head);
        self.tail.extend(other.tail);
        self
    }
}

impl<T> IntoIterator for NonEmptyVec<T> {
    type Item = T;
    type IntoIter = std::iter::Chain<std::iter::Once<T>, std::vec::IntoIter<T>>;

    fn into_iter(self) -> Self::IntoIter {
        std::iter::once(self.head).chain(self.tail)
    }
}

impl<'a, T> IntoIterator for &'a NonEmptyVec<T> {
    type Item = &'a T;
    type IntoIter = std::iter::Chain<std::iter::Once<&'a T>, std::slice::Iter<'a, T>>;

    fn into_iter(self) -> Self::IntoIter {
        std::iter::once(&self.head).chain(self.tail.iter())
    }
}

impl<T: fmt::Display> fmt::Display for NonEmptyVec<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, item) in self.iter().enumerate() {
            if i > 0 {
                writeln!(f)?;
            }
            write!(f, "- {}", item)?;
        }
        Ok(())
    }
}

/// Type alias for validation results
pub type ValidationResult<T> = Result<T, NonEmptyVec<ValidationError>>;

/// Helper to accumulate validation errors
///
/// Use this struct to collect multiple validation errors before
/// converting to a NonEmptyVec result.
pub struct ValidationAccumulator {
    errors: Vec<ValidationError>,
}

impl ValidationAccumulator {
    /// Create a new empty accumulator
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }

    /// Add an error to the accumulator
    pub fn add(&mut self, error: ValidationError) {
        self.errors.push(error);
    }

    /// Add an error if a condition is true
    pub fn add_if(&mut self, condition: bool, error: ValidationError) {
        if condition {
            self.errors.push(error);
        }
    }

    /// Check if there are any errors
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Get the number of accumulated errors
    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    /// Convert to a Result
    ///
    /// If there are no errors, returns Ok with the provided value.
    /// If there are errors, returns Err with the NonEmptyVec of errors.
    pub fn into_result<T>(self, value: T) -> ValidationResult<T> {
        match NonEmptyVec::from_vec(self.errors) {
            Some(errors) => Err(errors),
            None => Ok(value),
        }
    }

    /// Merge another accumulator's errors into this one
    pub fn merge(&mut self, other: ValidationAccumulator) {
        self.errors.extend(other.errors);
    }
}

impl Default for ValidationAccumulator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_error_display() {
        let error = ValidationError::new("name", "Name is required");
        assert_eq!(error.to_string(), "name: Name is required");

        let error_with_code = ValidationError::with_code("email", "Invalid", "INVALID_EMAIL");
        assert_eq!(error_with_code.to_string(), "[INVALID_EMAIL] email: Invalid");
    }

    #[test]
    fn test_non_empty_vec_from_vec() {
        let empty: Vec<i32> = vec![];
        assert!(NonEmptyVec::from_vec(empty).is_none());

        let single = vec![1];
        let nev = NonEmptyVec::from_vec(single).unwrap();
        assert_eq!(nev.len(), 1);
        assert!(nev.is_singleton());

        let multiple = vec![1, 2, 3];
        let nev = NonEmptyVec::from_vec(multiple).unwrap();
        assert_eq!(nev.len(), 3);
        assert!(!nev.is_singleton());
    }

    #[test]
    fn test_non_empty_vec_iteration() {
        let nev = NonEmptyVec::from_parts(1, vec![2, 3]);
        let collected: Vec<_> = nev.iter().copied().collect();
        assert_eq!(collected, vec![1, 2, 3]);
    }

    #[test]
    fn test_validation_accumulator() {
        let mut acc = ValidationAccumulator::new();
        assert!(!acc.has_errors());

        acc.add(ValidationError::required("name"));
        acc.add(ValidationError::invalid_email("email"));

        assert!(acc.has_errors());
        assert_eq!(acc.error_count(), 2);

        let result: ValidationResult<()> = acc.into_result(());
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 2);
    }

    #[test]
    fn test_error_factories() {
        let required = ValidationError::required("name");
        assert_eq!(required.code, Some("REQUIRED".to_string()));

        let invalid_email = ValidationError::invalid_email("email");
        assert_eq!(invalid_email.code, Some("INVALID_EMAIL".to_string()));

        let mismatch = ValidationError::mismatch("password", "password_confirm");
        assert_eq!(mismatch.code, Some("MISMATCH".to_string()));
    }
}
