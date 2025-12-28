//! Routing Primitives
//!
//! Core compositional routing primitives following arrow-based FRP.

use std::marker::PhantomData;

/// A route from input type A to output type B
///
/// Routes are pure functions with additional compositional structure.
/// They form a category with `id` and `>>>` (compose).
///
/// # Type Parameters
///
/// - `A`: Input type
/// - `B`: Output type
///
/// # Laws
///
/// Routes must satisfy categorical laws:
/// ```text
/// id >>> f = f                      (left identity)
/// f >>> id = f                      (right identity)
/// (f >>> g) >>> h = f >>> (g >>> h) (associativity)
/// ```
pub struct Route<A, B> {
    /// The function that implements this route
    function: Box<dyn Fn(A) -> B + Send + Sync>,
    _phantom: PhantomData<(A, B)>,
}

impl<A, B> Route<A, B>
where
    A: Send + Sync,
    B: Send + Sync,
{
    /// Create a route from a function
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use cim_keys::routing::Route;
    ///
    /// let double = Route::new(|x: i32| x * 2);
    /// assert_eq!(double.run(5), 10);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(A) -> B + Send + Sync + 'static,
    {
        Route {
            function: Box::new(f),
            _phantom: PhantomData,
        }
    }

    /// Run the route with an input value
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use cim_keys::routing::Route;
    ///
    /// let add_one = Route::new(|x: i32| x + 1);
    /// assert_eq!(add_one.run(5), 6);
    /// ```
    pub fn run(&self, input: A) -> B {
        (self.function)(input)
    }

    /// Sequential composition: self >>> other
    ///
    /// Creates a route that runs self, then runs other on the result.
    ///
    /// # Type Signature
    ///
    /// ```text
    /// (A → B) >>> (B → C) = (A → C)
    /// ```
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use cim_keys::routing::Route;
    ///
    /// let double = Route::new(|x: i32| x * 2);
    /// let add_one = Route::new(|x: i32| x + 1);
    ///
    /// let double_then_add = double.then(add_one);
    /// assert_eq!(double_then_add.run(5), 11); // (5 * 2) + 1
    /// ```
    pub fn then<C, F>(self, next: F) -> Route<A, C>
    where
        F: Fn(B) -> C + Send + Sync + 'static,
        A: 'static,
        B: Send + Sync + 'static,
        C: Send + Sync + 'static,
    {
        Route::new(move |a| next(self.run(a)))
    }

    /// Sequential composition using another Route
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use cim_keys::routing::Route;
    ///
    /// let double = Route::new(|x: i32| x * 2);
    /// let add_one = Route::new(|x: i32| x + 1);
    ///
    /// let composed = double.compose(add_one);
    /// assert_eq!(composed.run(5), 11);
    /// ```
    pub fn compose<C>(self, next: Route<B, C>) -> Route<A, C>
    where
        A: 'static,
        B: Send + Sync + 'static,
        C: Send + Sync + 'static,
    {
        Route::new(move |a| next.run(self.run(a)))
    }
}

/// Identity route: A → A
///
/// Returns its input unchanged. This is the identity morphism in the
/// route category.
///
/// # Laws
///
/// ```text
/// id >>> f = f  (left identity)
/// f >>> id = f  (right identity)
/// ```
///
/// # Example
///
/// ```rust,ignore
/// use cim_keys::routing::id;
///
/// let identity = id::<i32>();
/// assert_eq!(identity.run(42), 42);
/// ```
pub fn id<A>() -> Route<A, A>
where
    A: Send + Sync + 'static,
{
    Route::new(|a| a)
}

/// Sequential composition operator: f >>> g
///
/// Compose two routes sequentially. The output of f becomes the input to g.
///
/// # Type Signature
///
/// ```text
/// (A → B) >>> (B → C) = (A → C)
/// ```
///
/// # Example
///
/// ```rust,ignore
/// use cim_keys::routing::{Route, compose};
///
/// let double = Route::new(|x: i32| x * 2);
/// let add_one = Route::new(|x: i32| x + 1);
///
/// let workflow = compose(double, add_one);
/// assert_eq!(workflow.run(5), 11); // (5 * 2) + 1
/// ```
pub fn compose<A, B, C>(first: Route<A, B>, second: Route<B, C>) -> Route<A, C>
where
    A: Send + Sync + 'static,
    B: Send + Sync + 'static,
    C: Send + Sync + 'static,
{
    Route::new(move |a| second.run(first.run(a)))
}

/// Parallel composition operator: f *** g
///
/// Run two routes in parallel on a tuple input.
///
/// # Type Signature
///
/// ```text
/// (A → B) *** (C → D) = ((A, C) → (B, D))
/// ```
///
/// # Example
///
/// ```rust,ignore
/// use cim_keys::routing::{Route, parallel};
///
/// let double = Route::new(|x: i32| x * 2);
/// let negate = Route::new(|x: i32| -x);
///
/// let both = parallel(double, negate);
/// assert_eq!(both.run((5, 3)), (10, -3));
/// ```
pub fn parallel<A, B, C, D>(
    route1: Route<A, B>,
    route2: Route<C, D>,
) -> Route<(A, C), (B, D)>
where
    A: Send + Sync + 'static,
    B: Send + Sync + 'static,
    C: Send + Sync + 'static,
    D: Send + Sync + 'static,
{
    Route::new(move |(a, c): (A, C)| (route1.run(a), route2.run(c)))
}

/// Fanout operator: f &&& g
///
/// Apply both routes to the same input and return both results.
///
/// # Type Signature
///
/// ```text
/// (A → B) &&& (A → C) = (A → (B, C))
/// ```
///
/// # Example
///
/// ```rust,ignore
/// use cim_keys::routing::{Route, fanout};
///
/// let double = Route::new(|x: i32| x * 2);
/// let square = Route::new(|x: i32| x * x);
///
/// let both = fanout(double, square);
/// assert_eq!(both.run(5), (10, 25));
/// ```
pub fn fanout<A, B, C>(route1: Route<A, B>, route2: Route<A, C>) -> Route<A, (B, C)>
where
    A: Clone + Send + Sync + 'static,
    B: Send + Sync + 'static,
    C: Send + Sync + 'static,
{
    Route::new(move |a: A| {
        let b = route1.run(a.clone());
        let c = route2.run(a);
        (b, c)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_route() {
        let id_route = id::<i32>();
        assert_eq!(id_route.run(42), 42);
        assert_eq!(id_route.run(0), 0);
        assert_eq!(id_route.run(-10), -10);
    }

    #[test]
    fn test_route_creation() {
        let double = Route::new(|x: i32| x * 2);
        assert_eq!(double.run(5), 10);
        assert_eq!(double.run(0), 0);
        assert_eq!(double.run(-3), -6);
    }

    #[test]
    fn test_sequential_composition_then() {
        let double = Route::new(|x: i32| x * 2);
        let add_one = |x: i32| x + 1;

        let composed = double.then(add_one);
        assert_eq!(composed.run(5), 11); // (5 * 2) + 1
        assert_eq!(composed.run(0), 1);  // (0 * 2) + 1
    }

    #[test]
    fn test_sequential_composition_compose() {
        let double = Route::new(|x: i32| x * 2);
        let add_one = Route::new(|x: i32| x + 1);

        let composed = double.compose(add_one);
        assert_eq!(composed.run(5), 11);
    }

    #[test]
    fn test_compose_function() {
        let double = Route::new(|x: i32| x * 2);
        let add_one = Route::new(|x: i32| x + 1);

        let workflow = compose(double, add_one);
        assert_eq!(workflow.run(5), 11);
    }

    #[test]
    fn test_left_identity_law() {
        // id >>> f = f
        let double = Route::new(|x: i32| x * 2);
        let id_then_double = compose(id::<i32>(), Route::new(|x: i32| x * 2));

        assert_eq!(double.run(5), id_then_double.run(5));
        assert_eq!(double.run(10), id_then_double.run(10));
    }

    #[test]
    fn test_right_identity_law() {
        // f >>> id = f
        let double = Route::new(|x: i32| x * 2);
        let double_then_id = Route::new(|x: i32| x * 2).compose(id::<i32>());

        assert_eq!(double.run(5), double_then_id.run(5));
        assert_eq!(double.run(10), double_then_id.run(10));
    }

    #[test]
    fn test_associativity_law() {
        // (f >>> g) >>> h = f >>> (g >>> h)
        let left_assoc = compose(compose(
            Route::new(|x: i32| x * 2),
            Route::new(|x: i32| x + 1)
        ), Route::new(|x: i32| -x));

        let right_assoc = compose(
            Route::new(|x: i32| x * 2),
            compose(Route::new(|x: i32| x + 1), Route::new(|x: i32| -x))
        );

        assert_eq!(left_assoc.run(5), right_assoc.run(5));
        assert_eq!(left_assoc.run(10), right_assoc.run(10));
    }

    #[test]
    fn test_parallel_composition() {
        let double = Route::new(|x: i32| x * 2);
        let negate = Route::new(|x: i32| -x);

        let both = parallel(double, negate);
        assert_eq!(both.run((5, 3)), (10, -3));
        assert_eq!(both.run((0, 0)), (0, 0));
    }

    #[test]
    fn test_fanout() {
        let double = Route::new(|x: i32| x * 2);
        let square = Route::new(|x: i32| x * x);

        let both = fanout(double, square);
        assert_eq!(both.run(5), (10, 25));
        assert_eq!(both.run(3), (6, 9));
        assert_eq!(both.run(0), (0, 0));
    }

    #[test]
    fn test_complex_composition() {
        // Build a complex workflow: ((x * 2) + 1) then (negate, square)
        let double = Route::new(|x: i32| x * 2);
        let add_one = Route::new(|x: i32| x + 1);
        let negate = Route::new(|x: i32| -x);
        let square = Route::new(|x: i32| x * x);

        let workflow = compose(
            compose(double, add_one),
            fanout(negate, square)
        );

        // Input: 5
        // After double: 10
        // After add_one: 11
        // After fanout: (-11, 121)
        assert_eq!(workflow.run(5), (-11, 121));
    }

    #[test]
    fn test_string_routes() {
        let uppercase = Route::new(|s: String| s.to_uppercase());
        let add_exclamation = Route::new(|s: String| format!("{}!", s));

        let shout = compose(uppercase, add_exclamation);
        assert_eq!(shout.run("hello".into()), "HELLO!");
    }
}
