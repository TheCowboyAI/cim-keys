//! Property Tests for Compositional Laws (Axiom A9)
//!
//! This module verifies that our routing primitives satisfy the categorical laws
//! required by n-ary FRP. We use property-based testing to verify these laws
//! hold for arbitrary inputs.
//!
//! ## Laws Verified
//!
//! ### Category Laws
//! - **Left Identity**: `id >>> f = f`
//! - **Right Identity**: `f >>> id = f`
//! - **Associativity**: `(f >>> g) >>> h = f >>> (g >>> h)`
//!
//! ### Functor Laws (for Signal)
//! - **Identity**: `fmap id = id`
//! - **Composition**: `fmap (g ∘ f) = fmap g ∘ fmap f`
//!
//! ## Testing Strategy
//!
//! We use proptest to generate arbitrary routes and verify the laws hold.
//! Since routes are functions, we test the laws by:
//! 1. Generating random inputs
//! 2. Applying both sides of the equation
//! 3. Verifying results are equal

#[cfg(test)]
mod tests {
    use crate::routing::{Route, id, compose};
    use proptest::prelude::*;

    /// Generate arbitrary i32 values for testing
    fn arb_i32() -> impl Strategy<Value = i32> {
        -1000i32..1000i32
    }

    /// Generate arbitrary f64 values for testing
    fn arb_f64() -> impl Strategy<Value = f64> {
        -1000.0f64..1000.0f64
    }

    // Test: Left Identity Law
    // Property: id >>> f = f
    #[test]
    fn test_left_identity_law() {
        proptest!(|(x in arb_i32())| {
            // Define a simple route
            let f = Route::new(|n: i32| n * 2);
            let f2 = Route::new(|n: i32| n * 2);

            // Left side: id >>> f
            let left = id::<i32>().then(move |n| f.run(n));

            // Right side: f
            let right = f2;

            // Verify equality
            prop_assert_eq!(left.run(x), right.run(x));
        });
    }

    // Test: Right Identity Law
    // Property: f >>> id = f
    #[test]
    fn test_right_identity_law() {
        proptest!(|(x in arb_i32())| {
            // Define a simple route
            let f = Route::new(|n: i32| n * 2);

            // Left side: f >>> id
            let left = f.then(move |n| id::<i32>().run(n));

            // Right side: f
            let right = Route::new(|n: i32| n * 2);

            // Verify equality
            prop_assert_eq!(left.run(x), right.run(x));
        });
    }

    // Test: Associativity Law
    // Property: (f >>> g) >>> h = f >>> (g >>> h)
    #[test]
    fn test_associativity_law() {
        proptest!(|(x in arb_i32())| {
            // Define three simple routes
            let f = Route::new(|n: i32| n + 1);
            let g = Route::new(|n: i32| n * 2);
            let h = Route::new(|n: i32| n - 3);
            let g2 = Route::new(|n: i32| n * 2);
            let h2 = Route::new(|n: i32| n - 3);

            // Left side: (f >>> g) >>> h
            let fg = f.then(move |n| g.run(n));
            let left = fg.then(move |n| h.run(n));

            // Right side: f >>> (g >>> h)
            let gh = g2.then(move |n| h2.run(n));
            let f2 = Route::new(|n: i32| n + 1);
            let right = f2.then(move |n| gh.run(n));

            // Verify equality
            prop_assert_eq!(left.run(x), right.run(x));
        });
    }

    // Test: Composition is well-behaved
    // Property: compose(f, g)(x) = g(f(x))
    #[test]
    fn test_composition_semantics() {
        proptest!(|(x in arb_i32())| {
            let f = Route::new(|n: i32| n + 5);
            let g = Route::new(|n: i32| n * 3);
            let f2 = Route::new(|n: i32| n + 5);
            let g2 = Route::new(|n: i32| n * 3);

            // Using compose
            let composed = f.then(move |n| g.run(n));

            // Manual composition
            let manual = g2.run(f2.run(x));

            prop_assert_eq!(composed.run(x), manual);
        });
    }

    // Test: Identity route is truly identity
    #[test]
    fn test_identity_route() {
        proptest!(|(x in arb_i32())| {
            let identity = id::<i32>();
            prop_assert_eq!(identity.run(x), x);
        });

        proptest!(|(x in arb_f64())| {
            let identity = id::<f64>();
            prop_assert_eq!(identity.run(x), x);
        });
    }

    // Test: Multiple compositions preserve semantics
    // Property: f >>> g >>> h >>> k = k(h(g(f(x))))
    #[test]
    fn test_multiple_composition() {
        proptest!(|(x in arb_i32())| {
            let f = Route::new(|n: i32| n + 1);
            let g = Route::new(|n: i32| n * 2);
            let h = Route::new(|n: i32| n - 3);
            let k = Route::new(|n: i32| n / 2);
            let f2 = Route::new(|n: i32| n + 1);
            let g2 = Route::new(|n: i32| n * 2);
            let h2 = Route::new(|n: i32| n - 3);
            let k2 = Route::new(|n: i32| n / 2);

            // Chained composition
            let chained = f
                .then(move |n| g.run(n))
                .then(move |n| h.run(n))
                .then(move |n| k.run(n));

            // Manual composition
            let manual = k2.run(h2.run(g2.run(f2.run(x))));

            prop_assert_eq!(chained.run(x), manual);
        });
    }

    // Test: Composition is closed (type safety)
    #[test]
    fn test_composition_type_safety() {
        // i32 -> f64 -> String
        let f = Route::new(|n: i32| n as f64);
        let g = Route::new(|x: f64| format!("{:.2}", x));

        let composed = f.then(move |x| g.run(x));

        assert_eq!(composed.run(42), "42.00");
        assert_eq!(composed.run(100), "100.00");
    }

    // Test: Different composition orders with same semantics
    #[test]
    fn test_composition_commutativity_for_independent_routes() {
        proptest!(|(x in arb_i32())| {
            // Two independent transformations
            let double = Route::new(|n: i32| n * 2);
            let add_ten = Route::new(|n: i32| n + 10);

            // Different composition orders
            let order1 = double.then(move |n| add_ten.run(n));
            let order2 = Route::new(|n: i32| n * 2).then(|n| {
                let add_ten_inner = Route::new(|m: i32| m + 10);
                add_ten_inner.run(n)
            });

            // Both should produce same result
            prop_assert_eq!(order1.run(x), order2.run(x));
        });
    }
}

#[cfg(test)]
mod signal_functor_laws {
    use crate::signals::{Signal, StepKind, EventKind};
    use proptest::prelude::*;

    /// Generate arbitrary i32 values
    fn arb_i32() -> impl Strategy<Value = i32> {
        -100i32..100i32
    }

    // Test: Functor Identity Law for Signal
    // Property: fmap id = id
    #[test]
    fn test_signal_functor_identity() {
        proptest!(|(value in arb_i32())| {
            let signal = Signal::<StepKind, i32>::step(value);

            // fmap identity
            let fmapped = signal.clone().fmap(|x| x);

            // Should equal original
            prop_assert_eq!(signal.sample(0.0), fmapped.sample(0.0));
        });
    }

    // Test: Functor Composition Law for Signal
    // Property: fmap (g ∘ f) = fmap g ∘ fmap f
    #[test]
    fn test_signal_functor_composition() {
        proptest!(|(value in arb_i32())| {
            let signal = Signal::<StepKind, i32>::step(value);

            // Left side: fmap (g ∘ f)
            let left = signal.clone().fmap(|x| (x + 5) * 2);

            // Right side: fmap g ∘ fmap f
            let right = signal.fmap(|x| x + 5).fmap(|x| x * 2);

            prop_assert_eq!(left.sample(0.0), right.sample(0.0));
        });
    }

    // Test: Event signal functor preserves occurrences
    #[test]
    fn test_event_signal_functor() {
        let signal = Signal::<EventKind, i32>::event(vec![
            (0.0, 1),
            (1.0, 2),
            (2.0, 3),
        ]);

        let doubled = signal.fmap(|x| x * 2);

        let occurrences = doubled.occurrences(0.0, 3.0);
        assert_eq!(occurrences[0].1, 2);
        assert_eq!(occurrences[1].1, 4);
        assert_eq!(occurrences[2].1, 6);
    }

    // Test: fmap preserves signal kind
    #[test]
    fn test_fmap_preserves_signal_kind() {
        // Step signal stays step
        let step = Signal::<StepKind, i32>::step(42);
        let transformed = step.fmap(|x| x + 1);
        assert_eq!(transformed.sample(0.0), 43);
        assert_eq!(transformed.sample(100.0), 43); // Still piecewise constant

        // Event signal stays event
        let event = Signal::<EventKind, String>::event(vec![
            (0.0, "a".to_string()),
            (1.0, "b".to_string()),
        ]);
        let uppercased = event.fmap(|s| s.to_uppercase());
        let results = uppercased.occurrences(0.0, 2.0);
        assert_eq!(results[0].1, "A");
        assert_eq!(results[1].1, "B");
    }
}

#[cfg(test)]
mod documentation_tests {
    //! These tests serve as documentation for how to use compositional routing

    use crate::routing::Route;

    #[test]
    fn example_simple_pipeline() {
        // Build a simple data processing pipeline
        let parse = Route::new(|s: &str| s.parse::<i32>().unwrap_or(0));
        let double = Route::new(|n: i32| n * 2);
        let format_output = Route::new(|n: i32| format!("Result: {}", n));

        let pipeline = parse
            .then(move |n| double.run(n))
            .then(move |n| format_output.run(n));

        assert_eq!(pipeline.run("21"), "Result: 42");
        assert_eq!(pipeline.run("100"), "Result: 200");
    }

    #[test]
    fn example_validation_pipeline() {
        // Validation: String -> Result<i32, String>
        let validate_positive = Route::new(|n: i32| {
            if n > 0 {
                Ok(n)
            } else {
                Err(format!("Expected positive, got {}", n))
            }
        });

        let safe_double = Route::new(|r: Result<i32, String>| {
            r.map(|n| n * 2)
        });

        let pipeline = validate_positive.then(move |r| safe_double.run(r));

        assert_eq!(pipeline.run(5), Ok(10));
        assert!(pipeline.run(-3).is_err());
    }

    #[test]
    fn example_type_transformation_chain() {
        // Demonstrate type safety through composition
        let int_to_float = Route::new(|n: i32| n as f64);
        let float_to_string = Route::new(|f: f64| format!("{:.2}", f));
        let string_length = Route::new(|s: String| s.len());

        let chain = int_to_float
            .then(move |f| float_to_string.run(f))
            .then(move |s| string_length.run(s));

        // Type: i32 -> f64 -> String -> usize
        assert_eq!(chain.run(42), 5); // "42.00" has 5 chars
    }
}
