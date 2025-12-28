//! Route Builder API
//!
//! Provides a fluent builder interface for constructing complex routes
//! from simple primitives.
//!
//! # Example
//!
//! ```rust,ignore
//! use cim_keys::routing::RouteBuilder;
//!
//! let workflow = RouteBuilder::new()
//!     .then(validate_input)
//!     .then(process_data)
//!     .fanout(
//!         |data| save_to_disk(data),
//!         |data| send_to_nats(data)
//!     )
//!     .then(|(disk_result, nats_result)| merge_results(disk_result, nats_result))
//!     .build();
//! ```

use super::primitives::{Route, id, fanout};
use std::marker::PhantomData;

/// Builder for constructing complex routes using a fluent API
///
/// The builder maintains type safety throughout the construction process,
/// ensuring that incompatible routes cannot be composed.
///
/// # Type Parameters
///
/// - `A`: Input type for the route being built
/// - `B`: Output type for the route being built
pub struct RouteBuilder<A, B>
where
    A: Send + Sync + 'static,
    B: Send + Sync + 'static,
{
    route: Route<A, B>,
    _phantom: PhantomData<(A, B)>,
}

impl<A> RouteBuilder<A, A>
where
    A: Send + Sync + 'static,
{
    /// Create a new route builder starting with the identity route
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use cim_keys::routing::RouteBuilder;
    ///
    /// let builder = RouteBuilder::<i32, i32>::new();
    /// ```
    pub fn new() -> Self {
        RouteBuilder {
            route: id(),
            _phantom: PhantomData,
        }
    }
}

impl<A> Default for RouteBuilder<A, A>
where
    A: Send + Sync + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<A, B> RouteBuilder<A, B>
where
    A: Send + Sync + 'static,
    B: Send + Sync + 'static,
{
    /// Create a builder from an existing route
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use cim_keys::routing::{Route, RouteBuilder};
    ///
    /// let double = Route::new(|x: i32| x * 2);
    /// let builder = RouteBuilder::from_route(double);
    /// ```
    pub fn from_route(route: Route<A, B>) -> Self {
        RouteBuilder {
            route,
            _phantom: PhantomData,
        }
    }

    /// Add a function to the route pipeline
    ///
    /// This composes the current route with a new function, creating a
    /// new builder with the updated output type.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use cim_keys::routing::RouteBuilder;
    ///
    /// let workflow = RouteBuilder::new()
    ///     .then(|x: i32| x * 2)
    ///     .then(|x: i32| x + 1)
    ///     .build();
    ///
    /// assert_eq!(workflow.run(5), 11);
    /// ```
    pub fn then<C, F>(self, f: F) -> RouteBuilder<A, C>
    where
        F: Fn(B) -> C + Send + Sync + 'static,
        C: Send + Sync + 'static,
    {
        RouteBuilder {
            route: self.route.then(f),
            _phantom: PhantomData,
        }
    }

    /// Compose with another route
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use cim_keys::routing::{Route, RouteBuilder};
    ///
    /// let double = Route::new(|x: i32| x * 2);
    /// let add_one = Route::new(|x: i32| x + 1);
    ///
    /// let workflow = RouteBuilder::from_route(double)
    ///     .compose(add_one)
    ///     .build();
    ///
    /// assert_eq!(workflow.run(5), 11);
    /// ```
    pub fn compose<C>(self, next: Route<B, C>) -> RouteBuilder<A, C>
    where
        C: Send + Sync + 'static,
    {
        RouteBuilder {
            route: self.route.compose(next),
            _phantom: PhantomData,
        }
    }

    /// Apply the current route to both elements of a tuple in parallel
    ///
    /// This is useful when you have a tuple and want to apply the same
    /// transformation to both elements.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use cim_keys::routing::RouteBuilder;
    ///
    /// let double = RouteBuilder::new()
    ///     .then(|x: i32| x * 2)
    ///     .build();
    ///
    /// // Apply double to both elements
    /// let result = (5, 10);
    /// // Would need parallel application...
    /// ```
    pub fn both<C, D>(self) -> RouteBuilder<(A, C), (B, D)>
    where
        Self: Clone,
        C: Send + Sync + 'static,
        D: Send + Sync + 'static,
        B: Clone,
    {
        // This is a simplified version - full implementation would need
        // the parallel combinator properly integrated
        unimplemented!("both() requires Clone on Route - use parallel() instead")
    }

    /// Split the route output into two branches (fanout)
    ///
    /// Takes the output of the current route and feeds it to two different
    /// routes, returning a tuple of both results.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use cim_keys::routing::RouteBuilder;
    ///
    /// let workflow = RouteBuilder::new()
    ///     .then(|x: i32| x + 1)
    ///     .split(
    ///         |x: i32| x * 2,      // double it
    ///         |x: i32| x * x       // square it
    ///     )
    ///     .build();
    ///
    /// assert_eq!(workflow.run(5), (12, 36)); // (5+1)*2 = 12, (5+1)^2 = 36
    /// ```
    pub fn split<C, D, F, G>(self, left: F, right: G) -> RouteBuilder<A, (C, D)>
    where
        B: Clone,
        F: Fn(B) -> C + Send + Sync + 'static,
        G: Fn(B) -> D + Send + Sync + 'static,
        C: Send + Sync + 'static,
        D: Send + Sync + 'static,
    {
        let left_route = Route::new(left);
        let right_route = Route::new(right);
        let fanout_route = fanout(left_route, right_route);

        RouteBuilder {
            route: self.route.compose(fanout_route),
            _phantom: PhantomData,
        }
    }

    /// Map over the first element of a tuple output
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use cim_keys::routing::RouteBuilder;
    ///
    /// let workflow = RouteBuilder::new()
    ///     .then(|x: i32| (x, x * 2))
    ///     .map_first(|x: i32| x + 1)
    ///     .build();
    ///
    /// assert_eq!(workflow.run(5), (6, 10));
    /// ```
    pub fn map_first<B1, B2, B1New, F>(self, _f: F) -> RouteBuilder<A, (B1New, B2)>
    where
        B: Into<(B1, B2)>,
        F: Fn(B1) -> B1New + Send + Sync + 'static,
        B1: Send + Sync + 'static,
        B2: Send + Sync + 'static,
        B1New: Send + Sync + 'static,
    {
        unimplemented!("map_first requires tuple destructuring - use then() with closure")
    }

    /// Finalize the builder and return the constructed route
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use cim_keys::routing::RouteBuilder;
    ///
    /// let route = RouteBuilder::new()
    ///     .then(|x: i32| x * 2)
    ///     .then(|x: i32| x + 1)
    ///     .build();
    ///
    /// assert_eq!(route.run(5), 11);
    /// ```
    pub fn build(self) -> Route<A, B> {
        self.route
    }

    /// Run the route immediately with an input (convenience method)
    ///
    /// Equivalent to `builder.build().run(input)`
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use cim_keys::routing::RouteBuilder;
    ///
    /// let result = RouteBuilder::new()
    ///     .then(|x: i32| x * 2)
    ///     .then(|x: i32| x + 1)
    ///     .run_with(5);
    ///
    /// assert_eq!(result, 11);
    /// ```
    pub fn run_with(self, input: A) -> B {
        self.route.run(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_new() {
        let builder = RouteBuilder::<i32, i32>::new();
        let route = builder.build();
        assert_eq!(route.run(42), 42);
    }

    #[test]
    fn test_builder_then() {
        let route = RouteBuilder::new()
            .then(|x: i32| x * 2)
            .then(|x: i32| x + 1)
            .build();

        assert_eq!(route.run(5), 11);
    }

    #[test]
    fn test_builder_from_route() {
        let double = Route::new(|x: i32| x * 2);
        let route = RouteBuilder::from_route(double)
            .then(|x: i32| x + 1)
            .build();

        assert_eq!(route.run(5), 11);
    }

    #[test]
    fn test_builder_compose() {
        let double = Route::new(|x: i32| x * 2);
        let add_one = Route::new(|x: i32| x + 1);

        let route = RouteBuilder::from_route(double)
            .compose(add_one)
            .build();

        assert_eq!(route.run(5), 11);
    }

    #[test]
    fn test_builder_split() {
        let route = RouteBuilder::new()
            .then(|x: i32| x + 1)
            .split(
                |x: i32| x * 2,
                |x: i32| x * x
            )
            .build();

        assert_eq!(route.run(5), (12, 36));
    }

    #[test]
    fn test_builder_run_with() {
        let result = RouteBuilder::new()
            .then(|x: i32| x * 2)
            .then(|x: i32| x + 1)
            .run_with(5);

        assert_eq!(result, 11);
    }

    #[test]
    fn test_complex_builder_workflow() {
        // Simulate a data processing pipeline
        let workflow = RouteBuilder::new()
            .then(|x: i32| x * 2)        // double
            .then(|x: i32| x + 10)       // add 10
            .split(
                |x: i32| x / 2,          // halve
                |x: i32| x % 3           // mod 3
            )
            .build();

        let result = workflow.run(5);
        assert_eq!(result, (10, 2)); // (5*2+10)/2 = 10, (5*2+10)%3 = 2
    }

    #[test]
    fn test_string_workflow() {
        let workflow = RouteBuilder::new()
            .then(|s: String| s.to_uppercase())
            .then(|s: String| format!("{}!", s))
            .build();

        assert_eq!(workflow.run("hello".into()), "HELLO!");
    }
}
