//! MVI Routing Pattern Example
//!
//! Demonstrates how to use compositional routing to replace pattern matching
//! in MVI (Model-View-Intent) update functions.
//!
//! Traditional MVI uses pattern matching:
//! ```rust,ignore
//! fn update(model: Model, intent: Intent) -> Model {
//!     match intent {
//!         Intent::A => handle_a(model),
//!         Intent::B => handle_b(model),
//!         // ... 70 more variants
//!     }
//! }
//! ```
//!
//! With routing, we compose pure functions:
//! ```rust,ignore
//! let update_route = RouteBuilder::new()
//!     .then(route_intent)
//!     .then(apply_handler)
//!     .build();
//! ```

use cim_keys::routing::{Route, RouteBuilder};

// ============================================================================
// Simplified Model and Intent for Demonstration
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
struct Model {
    organization_name: String,
    person_name: String,
    counter: i32,
    status: String,
}

impl Model {
    fn new() -> Self {
        Model {
            organization_name: String::new(),
            person_name: String::new(),
            counter: 0,
            status: "initialized".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Intent {
    SetOrganizationName(String),
    SetPersonName(String),
    IncrementCounter,
    DecrementCounter,
    Reset,
}

// ============================================================================
// Traditional Pattern Matching Approach
// ============================================================================

fn traditional_update(model: Model, intent: Intent) -> Model {
    match intent {
        Intent::SetOrganizationName(name) => {
            let mut new_model = model;
            new_model.organization_name = name;
            new_model.status = "organization name updated".to_string();
            new_model
        }
        Intent::SetPersonName(name) => {
            let mut new_model = model;
            new_model.person_name = name;
            new_model.status = "person name updated".to_string();
            new_model
        }
        Intent::IncrementCounter => {
            let mut new_model = model;
            new_model.counter += 1;
            new_model.status = "counter incremented".to_string();
            new_model
        }
        Intent::DecrementCounter => {
            let mut new_model = model;
            new_model.counter -= 1;
            new_model.status = "counter decremented".to_string();
            new_model
        }
        Intent::Reset => Model::new(),
    }
}

// ============================================================================
// Routing-Based Approach
// ============================================================================

// Individual handler functions (pure, composable)
fn handle_set_org_name(name: String) -> impl Fn(Model) -> Model {
    move |mut model| {
        model.organization_name = name.clone();
        model.status = "organization name updated".to_string();
        model
    }
}

fn handle_set_person_name(name: String) -> impl Fn(Model) -> Model {
    move |mut model| {
        model.person_name = name.clone();
        model.status = "person name updated".to_string();
        model
    }
}

fn handle_increment_counter(model: Model) -> Model {
    let mut new_model = model;
    new_model.counter += 1;
    new_model.status = "counter incremented".to_string();
    new_model
}

fn handle_decrement_counter(model: Model) -> Model {
    let mut new_model = model;
    new_model.counter -= 1;
    new_model.status = "counter decremented".to_string();
    new_model
}

fn handle_reset(_model: Model) -> Model {
    Model::new()
}

// Route Intent to appropriate handler
fn route_intent((model, intent): (Model, Intent)) -> Model {
    match intent {
        Intent::SetOrganizationName(name) => {
            Route::new(handle_set_org_name(name)).run(model)
        }
        Intent::SetPersonName(name) => {
            Route::new(handle_set_person_name(name)).run(model)
        }
        Intent::IncrementCounter => {
            Route::new(handle_increment_counter).run(model)
        }
        Intent::DecrementCounter => {
            Route::new(handle_decrement_counter).run(model)
        }
        Intent::Reset => {
            Route::new(handle_reset).run(model)
        }
    }
}

// Routing-based update using RouteBuilder
fn routing_update(model: Model, intent: Intent) -> Model {
    RouteBuilder::new()
        .then(move |m: Model| (m, intent.clone()))
        .then(route_intent)
        .run_with(model)
}

// ============================================================================
// Example: Composing Multiple Updates
// ============================================================================

fn example_1_comparing_approaches() {
    println!("=== Example 1: Comparing Traditional vs Routing Approaches ===\n");

    let model = Model::new();

    // Traditional approach
    let model1 = traditional_update(model.clone(), Intent::SetOrganizationName("CowboyAI".to_string()));
    let model1 = traditional_update(model1, Intent::SetPersonName("Alice".to_string()));
    let model1 = traditional_update(model1, Intent::IncrementCounter);

    println!("Traditional result: {:?}", model1);

    // Routing approach
    let model2 = routing_update(model.clone(), Intent::SetOrganizationName("CowboyAI".to_string()));
    let model2 = routing_update(model2, Intent::SetPersonName("Alice".to_string()));
    let model2 = routing_update(model2, Intent::IncrementCounter);

    println!("Routing result:     {:?}", model2);
    println!("\nResults are equal: {}", model1 == model2);
    println!();
}

// ============================================================================
// Example: Pure Handler Composition
// ============================================================================

fn example_2_pure_handler_composition() {
    println!("=== Example 2: Pure Handler Composition ===\n");

    // Handlers can be composed independently of the update function
    let set_org = handle_set_org_name("Acme Corp".to_string());
    let increment = handle_increment_counter;

    let composed_handler = RouteBuilder::new()
        .then(set_org)
        .then(increment)
        .build();

    let model = Model::new();
    let result = composed_handler.run(model);

    println!("After composed handlers: {:?}", result);
    println!();
}

// ============================================================================
// Example: Branching Based on Model State
// ============================================================================

fn validate_model(model: Model) -> (Model, bool) {
    let is_valid = !model.organization_name.is_empty() && !model.person_name.is_empty();
    (model, is_valid)
}

fn process_if_valid((model, is_valid): (Model, bool)) -> Model {
    if is_valid {
        let mut new_model = model;
        new_model.status = "validated and processed".to_string();
        new_model
    } else {
        let mut new_model = model;
        new_model.status = "validation failed".to_string();
        new_model
    }
}

fn example_3_conditional_routing() {
    println!("=== Example 3: Conditional Processing with Routing ===\n");

    let validation_workflow = RouteBuilder::new()
        .then(validate_model)
        .then(process_if_valid)
        .build();

    // Invalid model (missing names)
    let model1 = Model::new();
    let result1 = validation_workflow.run(model1);
    println!("Invalid model result: status = {}", result1.status);

    // Valid model
    let mut model2 = Model::new();
    model2.organization_name = "CowboyAI".to_string();
    model2.person_name = "Alice".to_string();
    let result2 = validation_workflow.run(model2);
    println!("Valid model result: status = {}", result2.status);
    println!();
}

// ============================================================================
// Example: Workflow with Side Effects Tracking
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
struct ModelWithEffects {
    model: Model,
    effects: Vec<String>,
}

fn track_effect(effect: &str) -> impl Fn(Model) -> ModelWithEffects + '_ {
    move |model| {
        ModelWithEffects {
            model,
            effects: vec![effect.to_string()],
        }
    }
}

fn apply_and_track<'a>(handler: impl Fn(Model) -> Model + 'a, effect: &'a str) -> impl Fn(ModelWithEffects) -> ModelWithEffects + 'a {
    move |mwe| {
        let new_model = handler(mwe.model);
        let mut new_effects = mwe.effects.clone();
        new_effects.push(effect.to_string());
        ModelWithEffects {
            model: new_model,
            effects: new_effects,
        }
    }
}

fn example_4_effect_tracking() {
    println!("=== Example 4: Effect Tracking with Routing ===\n");

    let model = Model::new();

    // Track side effects through the pipeline
    let workflow = RouteBuilder::new()
        .then(track_effect("started"))
        .then(apply_and_track(
            handle_set_org_name("CowboyAI".to_string()),
            "set organization name"
        ))
        .then(apply_and_track(
            handle_increment_counter,
            "incremented counter"
        ))
        .build();

    let result = workflow.run(model);
    println!("Final model: {:?}", result.model);
    println!("Effects: {:?}", result.effects);
    println!();
}

// ============================================================================
// Main Example Runner
// ============================================================================

fn main() {
    println!("\n╔═══════════════════════════════════════════════════════════╗");
    println!("║  MVI Routing Pattern Examples                            ║");
    println!("║  Compositional update functions using routing DSL        ║");
    println!("╚═══════════════════════════════════════════════════════════╝\n");

    example_1_comparing_approaches();
    example_2_pure_handler_composition();
    example_3_conditional_routing();
    example_4_effect_tracking();

    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║  Key Insights:                                            ║");
    println!("║  - Handlers are pure, testable functions                 ║");
    println!("║  - Routes compose handlers without side effects          ║");
    println!("║  - Pattern matching isolated to routing layer            ║");
    println!("║  - Effects can be tracked through pipeline               ║");
    println!("╚═══════════════════════════════════════════════════════════╝\n");
}
