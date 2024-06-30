## rssafecircuit
This Rust library implements a Circuit Breaker pattern with asynchronous support using Tokio, managing failure states and recovery strategies.

### Components
- CircuitBreakerState:

- Enum defining the possible states of the circuit breaker:
  
  - Closed: Normal operation mode.
  - Open: Circuit breaker is open due to too many failures.
  - HalfOpen: Circuit breaker is in a trial mode to check if the underlying service has recovered.
- CircuitBreaker Struct:

  - Manages the state and behavior of the circuit breaker.
  - Tracks failures, successes, and transitions between states.

### Methods

**new(max_failures, timeout, pause_time):**

Constructor to initialize a new CircuitBreaker instance.

**Parameters:**
- `max_failures`: Maximum number of consecutive failures allowed before tripping.
- `timeout`: Duration in seconds for which the circuit breaker remains open.
- `pause_time`: Duration in milliseconds to wait before attempting to recheck the service.

**execute(func):**

Executes a given asynchronous function (`func`) wrapped in a future.
Handles the circuit breaker logic:
- Checks if the circuit is open or half-open before executing.
- Tracks successes and failures.
- Transitions state based on the number of failures.

**handle_failure():**

Increments failure counters and trips the circuit breaker if the threshold is reached.

**handle_success():**

Resets the circuit breaker state upon a successful operation.

**trip():**

Trips the circuit breaker, setting it to open state upon reaching the failure threshold.

**reset():**

Resets the circuit breaker to closed state upon recovery.

**set_on_open(callback):**

Sets a callback function to execute when the circuit breaker opens.

**set_on_close(callback):**

Sets a callback function to execute when the circuit breaker closes.

**set_on_half_open(callback):**

Sets a callback function to execute when the circuit breaker transitions to half-open state.

### Usage Example
Here’s an example demonstrating how to use the CircuitBreaker in a main.rs file using Tokio for asynchronous execution:
```rust
use std::time::Duration;
use tokio::time::sleep;
use tokio::runtime::Runtime;
use rssafecircuit::CircuitBreaker; // Adjust to match your crate name and structure

fn main() {
    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        let max_failures = 3;
        let timeout = 2;
        let pause_time = 1000;
        
        // Create a CircuitBreaker instance
        let mut breaker = CircuitBreaker::new(max_failures, timeout, pause_time);

        // Example operations
        let success_operation = || async { Ok("Operation successful".to_string()) };
        let failure_operation = || async { Err("Operation failed".to_string()) };

        // Example usage
        for _ in 0..=max_failures {
            let result = breaker.execute(success_operation.clone()).await;
            println!("Result: {:?}", result);
        }

        // Simulate max_failures+1 attempt to see the circuit breaker in action
        let result = breaker.execute(success_operation.clone()).await;
        println!("Result: {:?}", result);

        // Wait for the circuit breaker to potentially reset
        sleep(Duration::from_secs(timeout)).await;

        // Attempt after waiting, should be successful again
        let result = breaker.execute(success_operation.clone()).await;
        println!("Result: {:?}", result);

        // Trigger circuit breaker with failure attempts
        for _ in 0..=max_failures {
            let result = breaker.execute(failure_operation.clone()).await;
            println!("Result: {:?}", result);
        }

        // Another attempt to see circuit breaker in action
        let result = breaker.execute(failure_operation).await;
        println!("Result: {:?}", result);
    });
}

```

### How to Use

### Installation:

1. Ensure you have Rust and Cargo installed.
2. Add `rssafecircuit` (or your crate name) as a dependency in your `Cargo.toml`.

### Usage:

1. Create a CircuitBreaker instance with desired configuration parameters.
2. Define operations (success and failure scenarios) as asynchronous closures (async blocks).
3. Use the `execute()` method to run operations through the circuit breaker, observing state transitions and handling of failures.

### Callbacks (Optional):

- Set callbacks using `set_on_open()`, `set_on_close()`, and `set_on_half_open()` methods to perform actions on state changes (open, close, half-open).

## License
This project is licensed under the MIT License - see the LICENSE file for details.
