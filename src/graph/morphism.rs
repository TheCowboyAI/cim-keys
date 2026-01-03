// Copyright (c) 2025 - Cowboy AI, LLC.

//! Morphism Registry - Categorical Fold Infrastructure
//!
//! This module implements the MorphismRegistry pattern, which replaces
//! the 29-arm FoldDomainNode pattern matching with morphisms as DATA.
//!
//! # Mathematical Foundation
//!
//! For a coproduct `A₁ + A₂ + ... + Aₙ` (represented by `Injection` enum),
//! and morphisms `fᵢ: Aᵢ → B`, the universal property of coproducts
//! guarantees a unique morphism:
//!
//! ```text
//! [f₁, f₂, ..., fₙ]: A₁ + A₂ + ... + Aₙ → B
//! ```
//!
//! The `MorphismRegistry<B>` IS this unique morphism, represented as a
//! HashMap indexed by `Injection` (the coproduct tag).
//!
//! # FRP Axiom Compliance
//!
//! - **A5 (Totality)**: `CompleteMorphismRegistry<B>` guarantees all cases handled
//! - **A6 (Explicit Routing)**: No pattern matching at fold time - just lookup
//! - **A9 (Semantic Preservation)**: Functor laws verified by property tests
//!
//! # Usage
//!
//! ```rust,ignore
//! use cim_keys::graph::morphism::MorphismRegistry;
//! use cim_keys::lifting::Injection;
//!
//! // Register morphisms as DATA, not CODE branches
//! let registry = MorphismRegistry::<String>::new()
//!     .with::<Person>(|p| format!("Person: {}", p.name))
//!     .with::<Organization>(|o| format!("Org: {}", o.name))
//!     // ... 27 more registrations ...
//!     ;
//!
//! // Single fold operation - NO pattern matching
//! let result: Option<String> = registry.fold(&lifted_node);
//! ```
//!
//! # Kan Extension Pattern
//!
//! This implements the Kan extension `Lan_K F` where:
//! - `K: CatDomain → CatInjection` is the "forget structure" functor (type tag only)
//! - `F: CatDomain → CatTarget` is the domain-specific morphism
//! - `Lan_K F: CatInjection → CatTarget` extends F along K
//!
//! The registry lookup IS the Kan extension - it lifts the abstract graph
//! operation back to domain semantics only when needed.

use std::collections::HashMap;
use std::sync::Arc;

use crate::lifting::{Injection, LiftableDomain, LiftedNode};

// ============================================================================
// MORPHISM - Type-erased transformation from LiftedNode to target type
// ============================================================================

/// A type-erased morphism from `LiftedNode` to target type `B`.
///
/// This captures `∃A. (A → B)` as a closure over the downcast operation.
/// The domain type `A` is erased at construction time, leaving only the
/// application function.
///
/// ## Categorical Interpretation
///
/// Each `Morphism<B>` represents a morphism `fᵢ: Aᵢ → B` from one of the
/// coproduct summands to the common target type `B`.
pub struct Morphism<B> {
    /// The morphism closure, handling downcast and transformation
    apply_fn: Arc<dyn Fn(&LiftedNode) -> Option<B> + Send + Sync>,
}

impl<B> Morphism<B> {
    /// Create a new morphism from a domain-typed function.
    ///
    /// This is the injection morphism: given `f: A → B`, produce a morphism
    /// that can be applied to any `LiftedNode` (returning `None` if the
    /// downcast fails).
    ///
    /// ## Type Safety
    ///
    /// The `Injection` tag is checked by the registry lookup, so the downcast
    /// should always succeed when called through `MorphismRegistry::fold()`.
    pub fn from_domain<A, F>(f: F) -> Self
    where
        A: Clone + Send + Sync + 'static,
        F: Fn(&A) -> B + Send + Sync + 'static,
        B: 'static,
    {
        Morphism {
            apply_fn: Arc::new(move |node: &LiftedNode| {
                node.downcast::<A>().map(|domain_data| f(domain_data))
            }),
        }
    }

    /// Apply this morphism to a lifted node.
    ///
    /// Returns `None` if the downcast fails (type mismatch).
    /// In a correctly-typed registry, this should always succeed.
    #[inline]
    pub fn apply(&self, node: &LiftedNode) -> Option<B> {
        (self.apply_fn)(node)
    }
}

impl<B> Clone for Morphism<B> {
    fn clone(&self) -> Self {
        Morphism {
            apply_fn: Arc::clone(&self.apply_fn),
        }
    }
}

impl<B> std::fmt::Debug for Morphism<B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Morphism<{}>", std::any::type_name::<B>())
    }
}

// ============================================================================
// MORPHISM REGISTRY - The unique morphism from coproduct to target
// ============================================================================

/// A registry of morphisms indexed by `Injection` type tag.
///
/// This IS the unique morphism `[f₁, f₂, ..., fₙ]: ∐Aᵢ → B` guaranteed
/// by the universal property of coproducts.
///
/// ## Morphisms as DATA
///
/// Instead of:
/// ```rust,ignore
/// match injection {
///     Injection::Person => fold_person(p),      // CODE branch 1
///     Injection::Organization => fold_org(o),  // CODE branch 2
///     // ... 27 more CODE branches ...
/// }
/// ```
///
/// We have:
/// ```rust,ignore
/// let registry = MorphismRegistry::new()
///     .with::<Person>(fold_person)      // DATA entry 1
///     .with::<Organization>(fold_org)   // DATA entry 2
///     // ... 27 more DATA entries ...
///     ;
/// registry.fold(node)  // Single lookup, no branches
/// ```
///
/// ## Builder Pattern
///
/// The registry is built incrementally using the `with()` method,
/// which infers the `Injection` tag from the domain type.
#[derive(Clone)]
pub struct MorphismRegistry<B> {
    morphisms: HashMap<Injection, Morphism<B>>,
}

impl<B: 'static> MorphismRegistry<B> {
    /// Create a new empty morphism registry.
    pub fn new() -> Self {
        MorphismRegistry {
            morphisms: HashMap::new(),
        }
    }

    /// Register a morphism for a domain type.
    ///
    /// The `Injection` tag is inferred from the `LiftableDomain` implementation.
    /// This is the key innovation: morphisms are registered as DATA entries,
    /// not as CODE branches in a match statement.
    ///
    /// ## Example
    ///
    /// ```rust,ignore
    /// let registry = MorphismRegistry::<String>::new()
    ///     .with::<Person>(|p| p.name.clone())
    ///     .with::<Organization>(|o| o.display_name.clone());
    /// ```
    pub fn with<A, F>(mut self, f: F) -> Self
    where
        A: LiftableDomain + Clone + Send + Sync + 'static,
        F: Fn(&A) -> B + Send + Sync + 'static,
    {
        let injection = A::injection();
        self.morphisms.insert(injection, Morphism::from_domain::<A, F>(f));
        self
    }

    /// Register a morphism for a specific injection tag.
    ///
    /// Use this when the domain type doesn't implement `LiftableDomain`,
    /// or when you need to register multiple morphisms for related types.
    pub fn with_injection<A, F>(mut self, injection: Injection, f: F) -> Self
    where
        A: Clone + Send + Sync + 'static,
        F: Fn(&A) -> B + Send + Sync + 'static,
    {
        self.morphisms.insert(injection, Morphism::from_domain::<A, F>(f));
        self
    }

    /// Fold a lifted node using the registered morphism.
    ///
    /// This is the single fold operation - NO pattern matching required.
    /// Looks up the morphism by injection tag and applies it.
    ///
    /// Returns `None` if:
    /// - No morphism is registered for this injection type
    /// - The morphism's downcast fails (should not happen with correct registration)
    ///
    /// ## FRP A6 Compliance
    ///
    /// This is explicit routing through data lookup, not code branching.
    #[inline]
    pub fn fold(&self, node: &LiftedNode) -> Option<B> {
        self.morphisms
            .get(&node.injection)
            .and_then(|morphism| morphism.apply(node))
    }

    /// Check if a morphism is registered for an injection type.
    pub fn has_morphism(&self, injection: Injection) -> bool {
        self.morphisms.contains_key(&injection)
    }

    /// Get the number of registered morphisms.
    pub fn len(&self) -> usize {
        self.morphisms.len()
    }

    /// Check if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.morphisms.is_empty()
    }

    /// Get all registered injection types.
    pub fn registered_types(&self) -> Vec<Injection> {
        self.morphisms.keys().copied().collect()
    }
}

impl<B: 'static> Default for MorphismRegistry<B> {
    fn default() -> Self {
        Self::new()
    }
}

impl<B> std::fmt::Debug for MorphismRegistry<B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MorphismRegistry")
            .field("registered_types", &self.morphisms.keys().collect::<Vec<_>>())
            .field("count", &self.morphisms.len())
            .finish()
    }
}

// ============================================================================
// COMPLETE MORPHISM REGISTRY - Totality Guarantee (FRP A5)
// ============================================================================

/// A morphism registry that guarantees all injection types are covered.
///
/// This wrapper provides the FRP A5 (Totality) guarantee at the type level.
/// It can only be constructed from a `MorphismRegistry` that has all
/// injection types registered.
///
/// ## Usage
///
/// ```rust,ignore
/// let registry = MorphismRegistry::new()
///     .with::<Person>(|p| p.name.clone())
///     // ... all other types ...
///     ;
///
/// // This will fail if any injection type is missing
/// let complete = CompleteMorphismRegistry::try_from(registry)?;
///
/// // Now fold is guaranteed to succeed
/// let result: B = complete.fold(node);  // No Option!
/// ```
#[derive(Clone, Debug)]
pub struct CompleteMorphismRegistry<B> {
    inner: MorphismRegistry<B>,
}

impl<B: 'static + Clone> CompleteMorphismRegistry<B> {
    /// Try to create a complete registry from a partial one.
    ///
    /// Returns `Err` with the list of missing injection types if not all
    /// types are covered.
    pub fn try_from_registry(
        registry: MorphismRegistry<B>,
        required_injections: &[Injection],
    ) -> Result<Self, Vec<Injection>> {
        let missing: Vec<Injection> = required_injections
            .iter()
            .filter(|inj| !registry.has_morphism(**inj))
            .copied()
            .collect();

        if missing.is_empty() {
            Ok(CompleteMorphismRegistry { inner: registry })
        } else {
            Err(missing)
        }
    }

    /// Fold a lifted node - guaranteed to succeed.
    ///
    /// This is the total fold operation - since all injection types are
    /// covered, we can guarantee a result without `Option`.
    ///
    /// ## Panics
    ///
    /// Panics if the downcast fails, which indicates a bug in the
    /// LiftableDomain implementation (wrong injection tag).
    #[inline]
    pub fn fold(&self, node: &LiftedNode) -> B {
        self.inner
            .fold(node)
            .expect("CompleteMorphismRegistry: downcast failed - injection tag mismatch")
    }

    /// Get the inner registry for partial operations.
    pub fn inner(&self) -> &MorphismRegistry<B> {
        &self.inner
    }
}

// ============================================================================
// LAZY MORPHISM - Deferred lifting (Thunk pattern)
// ============================================================================

/// A lazy morphism that defers computation until needed.
///
/// This implements the thunk pattern `Lazy<A> = () -> A` for efficient
/// deferred lifting in graph traversals.
///
/// ## Usage
///
/// ```rust,ignore
/// // During graph traversal, we collect lazy morphisms
/// let lazy: LazyMorphism<String> = LazyMorphism::new(&node, &registry);
///
/// // Only force the thunk when domain semantics are needed
/// if at_boundary {
///     let result = lazy.force();
/// }
/// ```
pub struct LazyMorphism<B> {
    thunk: Arc<dyn Fn() -> Option<B> + Send + Sync>,
}

impl<B: 'static> LazyMorphism<B> {
    /// Create a new lazy morphism from a node and registry.
    ///
    /// The fold is not computed until `force()` is called.
    pub fn new(node: &LiftedNode, registry: &MorphismRegistry<B>) -> Self
    where
        B: Clone,
    {
        let node_clone = node.clone();
        let registry_clone = registry.clone();
        LazyMorphism {
            thunk: Arc::new(move || registry_clone.fold(&node_clone)),
        }
    }

    /// Force the lazy computation.
    ///
    /// This is the Kan extension boundary - we lift only when needed.
    #[inline]
    pub fn force(&self) -> Option<B> {
        (self.thunk)()
    }
}

impl<B> Clone for LazyMorphism<B> {
    fn clone(&self) -> Self {
        LazyMorphism {
            thunk: Arc::clone(&self.thunk),
        }
    }
}

impl<B> std::fmt::Debug for LazyMorphism<B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LazyMorphism<{}>", std::any::type_name::<B>())
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Organization, Person};
    use crate::domain::ids::BootstrapOrgId;

    /// Helper to create a test Person
    fn test_person(name: &str, email: &str) -> Person {
        let org_id = BootstrapOrgId::new();
        Person::new(name, email, org_id)
    }

    /// Helper to create a test Organization
    fn test_organization(name: &str, display_name: &str) -> Organization {
        Organization::new(name, display_name)
    }

    /// Test: Morphism registration and lookup
    #[test]
    fn test_morphism_registration() {
        let registry = MorphismRegistry::<String>::new()
            .with::<Person, _>(|p| format!("Person: {}", p.name))
            .with::<Organization, _>(|o| format!("Org: {}", o.name));

        assert_eq!(registry.len(), 2);
        assert!(registry.has_morphism(Injection::Person));
        assert!(registry.has_morphism(Injection::Organization));
        assert!(!registry.has_morphism(Injection::Location));
    }

    /// Test: Fold operation on Person
    #[test]
    fn test_fold_person() {
        let registry = MorphismRegistry::<String>::new()
            .with::<Person, _>(|p| format!("Name: {}", p.name));

        let person = test_person("Alice", "alice@example.com");

        let node = person.lift();
        let result = registry.fold(&node);

        assert_eq!(result, Some("Name: Alice".to_string()));
    }

    /// Test: Fold returns None for unregistered type
    #[test]
    fn test_fold_unregistered_type() {
        let registry = MorphismRegistry::<String>::new()
            .with::<Person, _>(|p| p.name.clone());

        let org = test_organization("TestOrg", "Test Organization");

        let node = org.lift();
        let result = registry.fold(&node);

        assert!(result.is_none());
    }

    /// Test: Functor identity law - fold(lift(x)) = f(x)
    #[test]
    fn test_functor_identity_law() {
        let get_name = |p: &Person| p.name.clone();

        let registry = MorphismRegistry::<String>::new()
            .with::<Person, _>(get_name);

        let person = test_person("Bob", "bob@example.com");

        // fold(lift(x)) = f(x)
        let lifted_result = registry.fold(&person.lift()).unwrap();
        let direct_result = get_name(&person);

        assert_eq!(lifted_result, direct_result);
    }

    /// Test: Morphism composition - fold with composed function
    #[test]
    fn test_morphism_composition() {
        // f: Person -> String (get name)
        // g: String -> usize (get length)
        // Composed: Person -> usize

        let registry = MorphismRegistry::<usize>::new()
            .with::<Person, _>(|p| p.name.len());

        let person = test_person("Charlie", "charlie@example.com");

        let result = registry.fold(&person.lift()).unwrap();
        assert_eq!(result, 7); // "Charlie".len() == 7
    }

    /// Test: LazyMorphism defers computation
    #[test]
    fn test_lazy_morphism() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        let call_count = Arc::new(AtomicUsize::new(0));
        let call_count_clone = Arc::clone(&call_count);

        let registry = MorphismRegistry::<String>::new()
            .with::<Person, _>(move |p| {
                call_count_clone.fetch_add(1, Ordering::SeqCst);
                p.name.clone()
            });

        let person = test_person("Deferred", "deferred@example.com");

        let node = person.lift();
        let lazy = LazyMorphism::new(&node, &registry);

        // Not called yet
        assert_eq!(call_count.load(Ordering::SeqCst), 0);

        // Force the thunk
        let result = lazy.force();
        assert_eq!(result, Some("Deferred".to_string()));
        assert_eq!(call_count.load(Ordering::SeqCst), 1);

        // Force again increments count
        let _ = lazy.force();
        assert_eq!(call_count.load(Ordering::SeqCst), 2);
    }

    /// Test: Registered types listing
    #[test]
    fn test_registered_types() {
        let registry = MorphismRegistry::<String>::new()
            .with::<Person, _>(|p| p.name.clone())
            .with::<Organization, _>(|o| o.name.clone());

        let types = registry.registered_types();
        assert!(types.contains(&Injection::Person));
        assert!(types.contains(&Injection::Organization));
        assert_eq!(types.len(), 2);
    }
}
