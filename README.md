## Ascolt

[![crates.io version](https://img.shields.io/crates/v/ascolt.svg)](https://crates.io/crates/ascolt)
[![docs.rs docs](https://docs.rs/ascolt/badge.svg)](https://docs.rs/ascolt)
[![CI build](https://github.com/sterrlia/ascolt/actions/workflows/rust.yml/badge.svg)](https://github.com/sterrlia/ascolt/actions)
[![Dependency Status](https://deps.rs/crate/ascolt/latest/status.svg)](https://deps.rs/crate/ascolt)
![Minimum Supported Rust Version](https://img.shields.io/badge/rustc-1.85.1+-ab6000.svg)

Async runtime-agnostic actor framework

### Features
+ Clean syntax for request-response pattern (you just implement `CallHandlerTrait`)
+ Async support
+ Handler errors can trigger actor stop, forced stop, or restart (behavior is customizable)
+ No built-in child actor concept, but you can run actors within other actors and store their `tx` in the parent actor's state
+ No heavy macros
+ Strict typing

### Examples
``` rust
let first_actor = FirstActor {};
let (first_actor_tx, first_actor_rx) = create_channel(100);
let second_actor = SecondActor { first_actor_tx };
let (second_actor_tx, second_actor_rx) = create_channel(100);

// if using tokio runtime
tokio::spawn(start_actor(actor, first_actor_rx));
tokio::spawn(start_actor(second_actor, second_actor_rx));

first_actor_tx.tell(SomeRequest { number: 3 })
    .await?; // fire-and-forget

let result = second_actor_tx
    .call(SecondActorCalcRequest(10))
    .await?; // request-response

println!("Result: {}", result.0);
```

More examples located at examples directory.

## What can be added
- Supervisor for controlling multiple actors
- Dependency graph (e.g., automatically shut down actors when the actors they depend on stop)
- Actor communication over the network
