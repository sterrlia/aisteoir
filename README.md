### Pros
+ Cleaner syntax for request-response pattern (you just implement CallHandlerTrait)
+ Async support
+ Handler errors can trigger actor stop, forced stop, or restart (behavior is customizable)
+ Was fun to develop
+ No built-in child actor concept, but you can run actors within other actors and store their tx in the parent actor's state
+ No heavy macros
+ Strict typing

### Cons
- No one will use it except me

### Examples
``` rust
let first_actor = FirstActor {};
let first_actor_tx = start_actor(actor, 100);

first_actor_tx.tell(FirstActorMessage::SomeRequest, SomeRequest { number: 3 })
    .await?; // fire and forget

let second_actor = ProxyActor { tx };
let second_actor_tx = start_actor(second_actor, 100);

let result = second_actor_tx
    .call(SecondActorMessage::Get, ProxyActorCalcRequest(10))
    .await?; // request-response

println!("Result: {}", result.0);
```

More examples located at examples directory.

## What can be added
- Supervisor for controlling multiple actors
- Dependency graph (e.g., automatically shut down actors when the actors they depend on stop)
- Actor communication over the network
