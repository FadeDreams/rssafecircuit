// Import necessary modules
use std::time::{Duration, Instant};
use tokio::sync::broadcast;
use tokio::time::sleep;

#[derive(Debug, PartialEq)]
pub enum CircuitBreakerState {
    Closed,
    Open,
    HalfOpen,
}

pub struct CircuitBreaker {
    pub state: CircuitBreakerState,
    pub consecutive_failures: u32,
    pub total_failures: u32,
    pub total_successes: u32,
    pub max_failures: u32,
    pub timeout: Duration,
    pub open_timeout: Instant,
    pub pause_time: Duration,
    pub consecutive_successes: u32,
    sender: broadcast::Sender<String>,
}

impl CircuitBreaker {
    pub fn new(max_failures: u32, timeout: u64, pause_time: u64) -> Self {
        let (sender, _receiver) = broadcast::channel(16);
        Self {
            state: CircuitBreakerState::Closed,
            consecutive_failures: 0,
            total_failures: 0,
            total_successes: 0,
            max_failures,
            timeout: Duration::from_secs(timeout),
            open_timeout: Instant::now(),
            pause_time: Duration::from_millis(pause_time),
            consecutive_successes: 0,
            sender,
        }
    }

    pub async fn execute<F, Fut>(&mut self, mut func: F) -> Result<String, String>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<String, String>>,
    {
        match self.state {
            CircuitBreakerState::Open => {
                if Instant::now() > self.open_timeout {
                    self.state = CircuitBreakerState::HalfOpen;
                    self.sender.send("halfOpen".to_string()).unwrap();
                } else {
                    return Err("Circuit breaker is open".to_string());
                }
            }
            CircuitBreakerState::HalfOpen => {
                let result = func().await;
                self.delay(self.pause_time).await;
                return result;
            }
            _ => {}
        }

        let result = func().await;
        match result {
            Ok(res) => {
                self.handle_success();
                Ok(res)
            }
            Err(err) => {
                self.handle_failure();
                Err(err)
            }
        }
    }

    pub fn handle_failure(&mut self) {
        self.consecutive_failures += 1;
        self.total_failures += 1;
        if self.consecutive_failures >= self.max_failures {
            self.trip();
        }
    }

    pub fn handle_success(&mut self) {
        self.reset();
        self.total_successes += 1;
    }

    pub fn trip(&mut self) {
        self.state = CircuitBreakerState::Open;
        self.consecutive_failures = 0;
        self.consecutive_successes = 0;
        self.open_timeout = Instant::now() + self.timeout;
        self.sender.send("open".to_string()).unwrap();
    }

    pub fn reset(&mut self) {
        self.state = CircuitBreakerState::Closed;
        self.consecutive_failures = 0;
        self.consecutive_successes = 0;

        // Handle potential error when sending "close"
        if let Err(err) = self.sender.send("close".to_string()) {
            eprintln!("Error sending 'close' message: {:?}", err);
            // Handle the error as needed, maybe retry or log it
        }
    }

    async fn delay(&self, duration: Duration) {
        sleep(duration).await;
    }

    pub fn set_on_open<F>(&self, mut callback: F)
    where
        F: FnMut() + Send + 'static,
    {
        let mut receiver = self.sender.subscribe();
        tokio::spawn(async move {
            while let Ok(message) = receiver.recv().await {
                if message == "open" {
                    callback();
                }
            }
        });
    }

    pub fn set_on_close<F>(&self, mut callback: F)
    where
        F: FnMut() + Send + 'static,
    {
        let mut receiver = self.sender.subscribe();
        tokio::spawn(async move {
            while let Ok(message) = receiver.recv().await {
                if message == "close" {
                    callback();
                }
            }
        });
    }

    pub fn set_on_half_open<F>(&self, mut callback: F)
    where
        F: FnMut() + Send + 'static,
    {
        let mut receiver = self.sender.subscribe();
        tokio::spawn(async move {
            while let Ok(message) = receiver.recv().await {
                if message == "halfOpen" {
                    callback();
                }
            }
        });
    }
}
