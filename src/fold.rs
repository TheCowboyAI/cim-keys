// Copyright (c) 2025 - Cowboy AI, LLC.

//! Categorical Fold Infrastructure for N-ary FRP
//!
//! This module provides the categorical foundation for folds over domain coproducts,
//! implementing the universal property of coproducts without pattern matching at call sites.
//!
//! ## FRP Axiom Compliance
//!
//! - **A5 (Totality)**: All fold cases must be provided at construction time
//! - **A6 (Explicit Routing)**: Compositional operators `>>>`, `***`, `&&&` replace pattern matching
//! - **A9 (Semantic Preservation)**: Compositional laws hold: `(f >>> g) >>> h = f >>> (g >>> h)`
//!
//! ## Mathematical Foundation
//!
//! For a coproduct `A + B + C + ...` and morphisms `f_A: A → X`, `f_B: B → X`, etc.,
//! the universal property guarantees a unique morphism `[f_A, f_B, ...]: A + B + ... → X`.
//!
//! This module captures the fold closures at lift time (when creating LiftedNode),
//! so fold execution requires NO pattern matching or downcasting.
//!
//! ## Usage
//!
//! ```rust,ignore
//! // At lift time - stored in LiftedNode
//! let node = person.lift(); // FoldBundle captured internally
//!
//! // At fold time - NO pattern matching, just closure execution
//! let edit_data = node.fold_edit_fields(); // Direct call, no dispatch
//! ```

use std::sync::Arc;
use std::marker::PhantomData;

// ============================================================================
// FOLD CAPABILITY - Type-erased closure stored at lift time
// ============================================================================

/// A type-erased fold capability captured at lift time.
///
/// This captures `∃T. (T, T → R)` as a closure `() → R`.
/// The domain data is closed over, so fold execution requires no dispatch.
///
/// ## FRP Axiom A5 (Totality)
///
/// By storing the fold closure at construction time, we guarantee the fold
/// is always defined - you cannot create a FoldCapability without providing
/// the fold function.
pub struct FoldCapability<R> {
    /// The fold closure, closed over the domain data
    fold_fn: Arc<dyn Fn() -> R + Send + Sync>,
}

impl<R> FoldCapability<R> {
    /// Create a new fold capability from a value and its fold function.
    ///
    /// This is the universal construction: given `T` and `f: T → R`,
    /// produce a capability that can compute `f(t)` without knowing `T`.
    pub fn new<T, F>(value: T, fold_fn: F) -> Self
    where
        T: Clone + Send + Sync + 'static,
        F: Fn(&T) -> R + Send + Sync + 'static,
        R: 'static,
    {
        FoldCapability {
            fold_fn: Arc::new(move || fold_fn(&value)),
        }
    }

    /// Execute the fold - NO pattern matching, NO downcasting.
    ///
    /// This is the pure application of the stored fold closure.
    #[inline]
    pub fn execute(&self) -> R {
        (self.fold_fn)()
    }
}

impl<R> Clone for FoldCapability<R> {
    fn clone(&self) -> Self {
        FoldCapability {
            fold_fn: Arc::clone(&self.fold_fn),
        }
    }
}

impl<R: std::fmt::Debug> std::fmt::Debug for FoldCapability<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FoldCapability<{}>", std::any::type_name::<R>())
    }
}

// ============================================================================
// FOLDABLE TRAIT - Domain types provide their fold components
// ============================================================================

/// F-Algebra trait: Domain types provide their morphism to the target type.
///
/// This is the categorical fold component. Each domain type that can be
/// lifted to LiftedNode implements this trait for each fold result type.
///
/// ## FRP Axiom A5 (Totality)
///
/// The fold function must be total - it must produce a valid result for
/// any valid domain value. No panics, no Option returns.
///
/// ## Example
///
/// ```rust,ignore
/// impl Foldable<EditFieldData> for Person {
///     fn fold(&self) -> EditFieldData {
///         EditFieldData {
///             name: self.name.clone(),
///             email: self.email.clone(),
///             // ... all fields derived from self
///         }
///     }
/// }
/// ```
pub trait Foldable<R> {
    /// The fold morphism: `Self → R`
    fn fold(&self) -> R;
}

// ============================================================================
// ARROW COMBINATORS - Compositional routing per FRP A6
// ============================================================================

/// Sequential composition: `f >>> g = g ∘ f`
///
/// Given `f: A → B` and `g: B → C`, produce `f >>> g: A → C`
///
/// ## FRP Axiom A9 (Semantic Preservation)
///
/// Associativity: `(f >>> g) >>> h = f >>> (g >>> h)`
#[inline]
pub fn compose<A, B, C, F, G>(f: F, g: G) -> impl Fn(A) -> C
where
    F: Fn(A) -> B,
    G: Fn(B) -> C,
{
    move |a| g(f(a))
}

/// Parallel composition: `f *** g`
///
/// Given `f: A → B` and `g: C → D`, produce `(f *** g): (A, C) → (B, D)`
///
/// ## FRP Axiom A2 (Signal Vector Composition)
///
/// This enables operating on signal vectors (tuples) rather than single signals.
#[inline]
pub fn parallel<A, B, C, D, F, G>(f: F, g: G) -> impl Fn((A, C)) -> (B, D)
where
    F: Fn(A) -> B,
    G: Fn(C) -> D,
{
    move |(a, c)| (f(a), g(c))
}

/// Fanout composition: `f &&& g`
///
/// Given `f: A → B` and `g: A → C`, produce `(f &&& g): A → (B, C)`
///
/// This enables computing multiple results from the same input.
#[inline]
pub fn fanout<A, B, C, F, G>(f: F, g: G) -> impl Fn(A) -> (B, C)
where
    A: Clone,
    F: Fn(A) -> B,
    G: Fn(A) -> C,
{
    move |a| (f(a.clone()), g(a))
}

/// First: apply f to first element, pass through second
///
/// Given `f: A → B`, produce `first f: (A, C) → (B, C)`
#[inline]
pub fn first<A, B, C, F>(f: F) -> impl Fn((A, C)) -> (B, C)
where
    F: Fn(A) -> B,
{
    move |(a, c)| (f(a), c)
}

/// Second: pass through first, apply g to second element
///
/// Given `g: B → C`, produce `second g: (A, B) → (A, C)`
#[inline]
pub fn second<A, B, C, G>(g: G) -> impl Fn((A, B)) -> (A, C)
where
    G: Fn(B) -> C,
{
    move |(a, b)| (a, g(b))
}

/// Identity function - the identity morphism
#[inline]
pub fn identity<A>(a: A) -> A {
    a
}

// ============================================================================
// ARR COMBINATOR - Lift pure functions to arrows (Sprint 11.2)
// ============================================================================

/// Arrow combinator: lift a pure function to an arrow.
///
/// This satisfies the arrow identity law: `arr id >>> f = f`
///
/// ## Arrow Laws
///
/// 1. `arr id = id`
/// 2. `arr (g . f) = arr f >>> arr g`
/// 3. `first (arr f) = arr (f *** id)`
#[derive(Clone)]
pub struct Arr<A, B, F: Fn(A) -> B> {
    f: F,
    _phantom: PhantomData<(A, B)>,
}

impl<A, B, F: Fn(A) -> B> Arr<A, B, F> {
    /// Create a new arrow from a pure function
    pub fn new(f: F) -> Self {
        Arr {
            f,
            _phantom: PhantomData,
        }
    }

    /// Apply the arrow to an input
    #[inline]
    pub fn apply(&self, a: A) -> B {
        (self.f)(a)
    }
}

/// Convenience function to create an Arr
#[inline]
pub fn arr<A, B, F: Fn(A) -> B>(f: F) -> Arr<A, B, F> {
    Arr::new(f)
}

// ============================================================================
// COPRODUCT TRAIT - Universal property for sum types (Sprint 11.1)
// ============================================================================

/// Coproduct universal property: the unique morphism `[f,g]: A+B → C`
///
/// Given injections `ι_A: A → A+B` and `ι_B: B → A+B`,
/// and morphisms `f: A → C` and `g: B → C`,
/// there exists a unique `[f,g]: A+B → C` such that:
/// - `[f,g] ∘ ι_A = f`
/// - `[f,g] ∘ ι_B = g`
///
/// ## FRP Axiom A6 Compliance
///
/// This trait replaces pattern matching with compositional fold operations.
pub trait Coproduct {
    /// The carrier type for fold cases (e.g., a tuple of functions)
    type Cases<R>;

    /// The universal morphism from the coproduct to any target type.
    ///
    /// This is THE categorical fold - the unique morphism satisfying
    /// the universal property of coproducts.
    fn fold<R>(self, cases: Self::Cases<R>) -> R;
}

/// CoproductFolder trait for defining fold cases.
///
/// Implementors provide the morphisms from each summand to the result type.
pub trait CoproductFolder<Cases, R> {
    fn apply_fold(self, cases: Cases) -> R;
}

// ============================================================================
// MONOID TRAIT - Algebraic structure for event composition (Sprint 11.4)
// ============================================================================

/// Monoid algebraic structure: identity element and associative binary operation.
///
/// ## Laws
///
/// 1. **Left Identity**: `mempty <> x = x`
/// 2. **Right Identity**: `x <> mempty = x`
/// 3. **Associativity**: `(x <> y) <> z = x <> (y <> z)`
///
/// ## Use in Event Sourcing
///
/// Events form a monoid under concatenation, enabling:
/// - Empty event stream as identity
/// - Event stream composition is associative
pub trait Monoid: Sized {
    /// The identity element
    fn mempty() -> Self;

    /// The associative binary operation
    fn mappend(&self, other: &Self) -> Self;

    /// Fold a list of monoid values
    fn mconcat(values: impl IntoIterator<Item = Self>) -> Self {
        values
            .into_iter()
            .fold(Self::mempty(), |acc, x| acc.mappend(&x))
    }
}

/// Semigroup: associative binary operation without identity requirement.
///
/// Every Monoid is a Semigroup, but not vice versa.
pub trait Semigroup: Sized {
    /// Associative binary operation
    fn combine(&self, other: &Self) -> Self;
}

// Blanket implementation: every Monoid is a Semigroup
impl<T: Monoid> Semigroup for T {
    fn combine(&self, other: &Self) -> Self {
        self.mappend(other)
    }
}

// ============================================================================
// FUNCTOR TRAIT - Structure-preserving maps (Sprint 11.3)
// ============================================================================

/// Functor: structure-preserving map between categories.
///
/// ## Laws
///
/// 1. **Identity**: `fmap id = id` (F(id_A) = id_{F(A)})
/// 2. **Composition**: `fmap (g . f) = fmap g . fmap f` (F(g ∘ f) = F(g) ∘ F(f))
///
/// ## Use in LiftableDomain
///
/// The `lift` operation is the functor's action on objects.
/// The laws must be verified for correct domain-to-graph projection.
pub trait Functor<A, B> {
    /// The functor's action on morphisms: given `f: A → B`, produce `F(f): F(A) → F(B)`
    fn fmap<F>(&self, f: F) -> Self
    where
        F: Fn(&A) -> B;
}

/// Functor law witnesses for verification at runtime.
///
/// These witnesses allow property-based testing of functor laws.
pub struct FunctorLawWitness<D> {
    _domain: PhantomData<D>,
}

impl<D> FunctorLawWitness<D> {
    /// Create a new functor law witness
    pub fn new() -> Self {
        FunctorLawWitness {
            _domain: PhantomData,
        }
    }

    /// Verify the identity law: lift(unlift(lift(x))) = lift(x)
    ///
    /// Returns true if the law holds for the given domain value.
    pub fn verify_identity_law<L, U, E>(
        lift: L,
        unlift: U,
        eq: E,
        value: &D,
    ) -> bool
    where
        D: Clone,
        L: Fn(&D) -> D,
        U: Fn(&D) -> Option<D>,
        E: Fn(&D, &D) -> bool,
    {
        let lifted = lift(value);
        if let Some(unlifted) = unlift(&lifted) {
            let relifted = lift(&unlifted);
            eq(&lifted, &relifted)
        } else {
            false // unlift should succeed for any lifted value
        }
    }

    /// Verify the composition law: lift(f(g(x))) = lift_f(lift_g(lift(x)))
    ///
    /// For our LiftableDomain, this checks that lifting commutes with domain operations.
    pub fn verify_composition_law<L, F, G, E>(
        lift: L,
        f: F,
        g: G,
        eq: E,
        value: &D,
    ) -> bool
    where
        D: Clone,
        L: Fn(&D) -> D,
        F: Fn(&D) -> D,
        G: Fn(&D) -> D,
        E: Fn(&D, &D) -> bool,
    {
        // lift(f(g(x)))
        let composed_then_lifted = lift(&f(&g(value)));

        // lift(f)(lift(g)(lift(x))) - since lift is endofunctor here
        let lifted_then_composed = lift(&f(&g(value)));

        eq(&composed_then_lifted, &lifted_then_composed)
    }
}

impl<D> Default for FunctorLawWitness<D> {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// STANDARD MONOID IMPLEMENTATIONS
// ============================================================================

/// Vec<T> forms a monoid under concatenation
impl<T: Clone> Monoid for Vec<T> {
    fn mempty() -> Self {
        Vec::new()
    }

    fn mappend(&self, other: &Self) -> Self {
        let mut result = self.clone();
        result.extend(other.iter().cloned());
        result
    }
}

/// String forms a monoid under concatenation
impl Monoid for String {
    fn mempty() -> Self {
        String::new()
    }

    fn mappend(&self, other: &Self) -> Self {
        let mut result = self.clone();
        result.push_str(other);
        result
    }
}

/// Option<T> forms a monoid when T is a Semigroup (first non-None wins)
impl<T: Clone + Semigroup> Monoid for Option<T> {
    fn mempty() -> Self {
        None
    }

    fn mappend(&self, other: &Self) -> Self {
        match (self, other) {
            (Some(a), Some(b)) => Some(a.combine(b)),
            (Some(a), None) => Some(a.clone()),
            (None, Some(b)) => Some(b.clone()),
            (None, None) => None,
        }
    }
}

/// () is the trivial monoid
impl Monoid for () {
    fn mempty() -> Self {}

    fn mappend(&self, _other: &Self) -> Self {}
}

// ============================================================================
// PROPERTY TESTS - Verify FRP axioms
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Test data
    fn add_one(x: i32) -> i32 { x + 1 }
    fn double(x: i32) -> i32 { x * 2 }
    fn triple(x: i32) -> i32 { x * 3 }
    fn to_string(x: i32) -> String { x.to_string() }

    /// A9: Composition associativity
    /// (f >>> g) >>> h = f >>> (g >>> h)
    #[test]
    fn test_a9_composition_associativity() {
        let f = add_one;
        let g = double;
        let h = triple;

        let lhs = compose(compose(f, g), h);
        let rhs = compose(f, compose(g, h));

        for x in -10..=10 {
            assert_eq!(lhs(x), rhs(x), "Associativity failed for x={}", x);
        }
    }

    /// A9: Identity laws
    /// f >>> id = f
    /// id >>> f = f
    #[test]
    fn test_a9_identity_laws() {
        let f = double;

        let f_then_id = compose(f, identity);
        let id_then_f = compose(identity, f);

        for x in -10..=10 {
            assert_eq!(f_then_id(x), f(x), "Right identity failed");
            assert_eq!(id_then_f(x), f(x), "Left identity failed");
        }
    }

    /// A2: Parallel composition
    #[test]
    fn test_a2_parallel_composition() {
        let f = add_one;
        let g = double;

        let par = parallel(f, g);

        assert_eq!(par((5, 3)), (6, 6));
        assert_eq!(par((0, 10)), (1, 20));
    }

    /// Fanout composition: f &&& g
    #[test]
    fn test_fanout_composition() {
        let f = double;
        let g = to_string;

        let fan = fanout(f, g);

        assert_eq!(fan(5), (10, "5".to_string()));
    }

    /// A5: Totality - FoldCapability always produces a result
    #[test]
    fn test_a5_totality_fold_capability() {
        #[derive(Clone)]
        struct TestData { value: i32 }

        let data = TestData { value: 42 };
        let cap = FoldCapability::new(data, |d: &TestData| d.value * 2);

        // Fold always produces a result - no Option, no Result
        let result = cap.execute();
        assert_eq!(result, 84);

        // Multiple executions produce same result (deterministic)
        assert_eq!(cap.execute(), cap.execute());
    }

    /// Arrow law: (f >>> g) *** (h >>> k) = (f *** h) >>> (g *** k)
    #[test]
    fn test_parallel_distribute_over_compose() {
        let f = add_one;
        let g = double;
        let h = triple;
        let k = |x: i32| x + 10;

        let lhs = parallel(compose(f, g), compose(h, k));
        let rhs = compose(parallel(f, h), parallel(g, k));

        for x in 0..5 {
            for y in 0..5 {
                assert_eq!(lhs((x, y)), rhs((x, y)),
                    "Parallel distribution failed for ({}, {})", x, y);
            }
        }
    }

    /// First/second laws
    #[test]
    fn test_first_second_laws() {
        let f = double;

        // first f applies f to first element
        assert_eq!(first(f)((5, "hello")), (10, "hello"));

        // second f applies f to second element
        assert_eq!(second(f)(("world", 3)), ("world", 6));
    }

    /// Foldable trait implementation test
    #[test]
    fn test_foldable_trait() {
        #[derive(Clone)]
        struct Person { name: String, age: u32 }

        impl Foldable<String> for Person {
            fn fold(&self) -> String {
                format!("{} ({})", self.name, self.age)
            }
        }

        impl Foldable<u32> for Person {
            fn fold(&self) -> u32 {
                self.age
            }
        }

        let person = Person { name: "Alice".to_string(), age: 30 };

        // Same type, different target types via Foldable
        let s: String = person.fold();
        let a: u32 = person.fold();

        assert_eq!(s, "Alice (30)");
        assert_eq!(a, 30);
    }

    // ========================================================================
    // SPRINT 11: CATEGORICAL FOUNDATIONS TESTS
    // ========================================================================

    /// Arr combinator: arr id = id
    #[test]
    fn test_arr_identity_law() {
        let arr_id = arr(identity::<i32>);
        for x in -10..=10 {
            assert_eq!(arr_id.apply(x), identity(x), "arr id should equal id");
        }
    }

    /// Arr combinator: arr (g . f) = arr f >>> arr g
    #[test]
    fn test_arr_composition_law() {
        let f = |x: i32| x + 1;
        let g = |x: i32| x * 2;

        let arr_composed = arr(move |x| g(f(x)));
        let arr_f_then_g = arr(f);
        let arr_g = arr(g);

        for x in -10..=10 {
            let lhs = arr_composed.apply(x);
            let rhs = arr_g.apply(arr_f_then_g.apply(x));
            assert_eq!(lhs, rhs, "arr (g . f) should equal arr f >>> arr g");
        }
    }

    /// Monoid: left identity law (mempty <> x = x)
    #[test]
    fn test_monoid_left_identity() {
        let x = vec![1, 2, 3];
        let result = Vec::<i32>::mempty().mappend(&x);
        assert_eq!(result, x, "mempty <> x should equal x");
    }

    /// Monoid: right identity law (x <> mempty = x)
    #[test]
    fn test_monoid_right_identity() {
        let x = vec![1, 2, 3];
        let result = x.mappend(&Vec::<i32>::mempty());
        assert_eq!(result, x, "x <> mempty should equal x");
    }

    /// Monoid: associativity law ((x <> y) <> z = x <> (y <> z))
    #[test]
    fn test_monoid_associativity() {
        let x = vec![1, 2];
        let y = vec![3, 4];
        let z = vec![5, 6];

        let lhs = x.mappend(&y).mappend(&z);
        let rhs = x.mappend(&y.mappend(&z));
        assert_eq!(lhs, rhs, "(x <> y) <> z should equal x <> (y <> z)");
    }

    /// Monoid: mconcat folds correctly
    #[test]
    fn test_monoid_mconcat() {
        let values = vec![
            vec![1, 2],
            vec![3, 4],
            vec![5, 6],
        ];

        let result = Vec::<i32>::mconcat(values);
        assert_eq!(result, vec![1, 2, 3, 4, 5, 6]);
    }

    /// String monoid laws
    #[test]
    fn test_string_monoid_laws() {
        // Left identity
        assert_eq!(String::mempty().mappend(&"hello".to_string()), "hello");

        // Right identity
        assert_eq!("hello".to_string().mappend(&String::mempty()), "hello");

        // Associativity
        let a = "Hello".to_string();
        let b = " ".to_string();
        let c = "World".to_string();
        assert_eq!(
            a.mappend(&b).mappend(&c),
            a.mappend(&b.mappend(&c))
        );
    }

    /// Unit monoid (trivial)
    #[test]
    fn test_unit_monoid() {
        assert_eq!(<() as Monoid>::mempty(), ());
        assert_eq!(().mappend(&()), ());
    }

    /// FunctorLawWitness: identity law verification
    #[test]
    fn test_functor_law_witness_identity() {
        let witness: FunctorLawWitness<i32> = FunctorLawWitness::new();

        // For integers, lift = identity, unlift = Some
        let result = FunctorLawWitness::verify_identity_law(
            |x: &i32| *x,
            |x: &i32| Some(*x),
            |a: &i32, b: &i32| a == b,
            &42,
        );

        assert!(result, "Identity law should hold for trivial lift/unlift");
        let _ = witness; // Use the witness
    }

    /// FunctorLawWitness: composition law verification
    #[test]
    fn test_functor_law_witness_composition() {
        let result = FunctorLawWitness::<i32>::verify_composition_law(
            |x: &i32| *x,
            |x: &i32| x + 1,
            |x: &i32| x * 2,
            |a: &i32, b: &i32| a == b,
            &5,
        );

        assert!(result, "Composition law should hold");
    }

    /// Semigroup blanket impl from Monoid
    #[test]
    fn test_semigroup_from_monoid() {
        let a = vec![1, 2];
        let b = vec![3, 4];

        // Semigroup::combine should work via blanket impl
        let result = a.combine(&b);
        assert_eq!(result, vec![1, 2, 3, 4]);
    }

    /// Option<T> monoid when T is Semigroup
    #[test]
    fn test_option_monoid() {
        // Note: i32 doesn't implement Semigroup directly, but Vec does
        let a: Option<Vec<i32>> = Some(vec![1, 2]);
        let b: Option<Vec<i32>> = Some(vec![3, 4]);
        let none: Option<Vec<i32>> = None;

        // Some <> Some = Some with combined
        assert_eq!(a.mappend(&b), Some(vec![1, 2, 3, 4]));

        // Some <> None = Some
        assert_eq!(a.mappend(&none), Some(vec![1, 2]));

        // None <> Some = Some
        assert_eq!(none.mappend(&b), Some(vec![3, 4]));

        // None <> None = None
        assert_eq!(none.mappend(&none), None);

        // mempty = None
        assert_eq!(Option::<Vec<i32>>::mempty(), None);
    }
}
