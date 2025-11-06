# The Agency - Test Suite

This directory contains the comprehensive test suite for The Agency framework, including unit tests, integration tests, property-based tests, and BDD tests.

## Test Organization

### Unit Tests (`unit_tests.rs`)
- Core functionality tests
- Individual module testing
- Fast execution
- No external dependencies

### Integration Tests

#### LLM Provider Tests (`llm_tests.rs`)
- Multi-provider LLM integration
- OpenAI, Anthropic, Google, Groq, Together AI, Azure, Ollama
- Provider fallback mechanisms
- Embedding generation

#### A2A Communication Tests (`a2a_tests.rs`)
- Agent-to-agent messaging
- Service discovery
- Health monitoring
- Multi-protocol support (HTTP, WebSocket, Redis, RabbitMQ)

#### Unified Storage Tests (`unified_storage_tests.rs`)
- Workflow suspension/resumption
- Memory management
- Trace collection
- Evaluation datasets
- Multi-tenant data isolation

#### DateTime & Location Tools Tests (`datetime_location_tools_tests.rs`)
- Built-in tool functionality
- System information gathering
- Datetime operations
- Location services

### BDD Tests (Behavior-Driven Development)

#### `bdd_steps.rs`
Step definitions for agent capabilities feature tests:
- Basic conversation
- Memory storage and retrieval
- Tool usage
- Multi-step reasoning
- Error handling

#### `llm_provider_bdd.rs`
Step definitions for multi-provider LLM feature tests:
- Provider configuration
- Model selection
- Fallback logic
- Task-based routing

#### `unified_storage_bdd.rs`
Step definitions for unified storage feature tests:
- Data persistence
- Resource isolation
- Cross-component integration

### Property-Based Tests

#### `unified_storage_property_tests.rs`
- Fuzzing and property-based testing using PropTest
- Validates invariants across random inputs
- Tests edge cases automatically
- Regression test tracking

### Performance Tests

#### `unified_storage_benchmarks.rs`
- Performance benchmarks
- Throughput testing
- Latency measurements
- Scalability validation

## Running Tests

### Run All Tests
```bash
cargo test
```

### Run Specific Test Modules
```bash
# Unit tests only
cargo test --test unit_tests

# LLM provider tests
cargo test --test llm_tests

# A2A communication tests
cargo test --test a2a_tests

# Unified storage tests
cargo test --test unified_storage_tests

# Property-based tests
cargo test --test unified_storage_property_tests

# BDD tests
cargo test --test cucumber_runner
```

### Run Tests with Output
```bash
cargo test -- --nocapture
```

### Run Tests in Release Mode (faster)
```bash
cargo test --release
```

### Run Specific Test
```bash
cargo test test_name
```

### Run Tests Matching Pattern
```bash
cargo test storage
```

## BDD/Cucumber Tests

### Prerequisites
Cucumber tests require the feature files to exist in the `features/` directory.

### Run BDD Tests
```bash
# Run all BDD tests
cargo test --test cucumber_runner

# Run with detailed output
cargo test --test cucumber_runner -- --nocapture
```

### Environment Variables for BDD Tests

LLM Provider tests require API keys:
```bash
export OPENAI_API_KEY="sk-..."
export ANTHROPIC_API_KEY="sk-ant-..."
export GOOGLE_API_KEY="..."
export GROQ_API_KEY="gsk_..."
export TOGETHER_API_KEY="..."

# For Ollama tests
export OLLAMA_RUNNING="true"
```

A2A tests may require:
```bash
# Redis (if testing Redis transport)
export REDIS_URL="redis://localhost:6379"

# RabbitMQ (if testing RabbitMQ transport)
export RABBITMQ_URL="amqp://localhost:5672"
```

## Test Coverage

### Generate Coverage Report
```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage
cargo tarpaulin --out Html --output-dir coverage
```

View the report at `coverage/index.html`.

## Writing Tests

### Unit Test Example
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }

    #[tokio::test]
    async fn test_async_example() {
        let result = async_function().await;
        assert!(result.is_ok());
    }
}
```

### BDD Step Definition Example
```rust
use cucumber::{given, when, then, World};

#[given("a precondition")]
async fn setup_precondition(world: &mut MyWorld) {
    world.setup();
}

#[when(regex = r"I perform action (.*)")]
async fn perform_action(world: &mut MyWorld, action: String) {
    world.execute(&action).await;
}

#[then("the result should be correct")]
async fn verify_result(world: &mut MyWorld) {
    assert!(world.result.is_some());
}
```

### Property-Based Test Example
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_property(x in 0..100, y in 0..100) {
        assert!(x + y >= x);
        assert!(x + y >= y);
    }
}
```

## Mocking

### Using Mockall
```rust
use mockall::predicate::*;
use mockall::*;

#[automock]
trait MyTrait {
    fn method(&self, arg: i32) -> String;
}

#[test]
fn test_with_mock() {
    let mut mock = MockMyTrait::new();
    mock.expect_method()
        .with(eq(42))
        .returning(|_| "success".to_string());
    
    assert_eq!(mock.method(42), "success");
}
```

### Using WireMock (HTTP)
```rust
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path};

#[tokio::test]
async fn test_http_client() {
    let mock_server = MockServer::start().await;
    
    Mock::given(method("GET"))
        .and(path("/api/endpoint"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&mock_server)
        .await;
    
    // Test your HTTP client against mock_server.uri()
}
```

## Continuous Integration

### GitHub Actions
Tests run automatically on push/PR via GitHub Actions.

Configuration file: `.github/workflows/tests.yml`

### Test Matrix
- Rust versions: stable, nightly
- OS: Ubuntu, macOS, Windows
- Features: with/without optional features

## Test Organization Best Practices

1. **Keep tests close to code**: Use inline `#[cfg(test)]` modules for unit tests
2. **Use descriptive names**: `test_workflow_pause_resume_preserves_state`
3. **Test one thing**: Each test should verify a single behavior
4. **Use fixtures**: Create reusable test data and setup helpers
5. **Mock external dependencies**: Don't rely on external services in tests
6. **Clean up**: Ensure tests clean up resources (temp files, DB connections)

## Test Categories

| Category           | Files                           | Purpose                        |
|--------------------|---------------------------------|--------------------------------|
| Unit               | `unit_tests.rs`                 | Core functionality             |
| Integration        | `*_tests.rs`                    | Component interaction          |
| BDD                | `*_bdd.rs`, `features/*.feature`| Behavior specification         |
| Property           | `*_property_tests.rs`           | Invariant validation           |
| Benchmark          | `*_benchmarks.rs`               | Performance measurement        |

## Feature Coverage

| Feature                    | Unit Tests | Integration Tests | BDD Tests | Property Tests |
|----------------------------|------------|-------------------|-----------|----------------|
| Core Agent                 | ✅         | ✅                | ✅        | ⬜             |
| Multi-Provider LLM         | ✅         | ✅                | ✅        | ⬜             |
| A2A Communication          | ✅         | ✅                | ⬜        | ⬜             |
| Workflow Engine            | ✅         | ✅                | ⬜        | ⬜             |
| Multi-Agent Organizations  | ✅         | ✅                | ⬜        | ⬜             |
| Unified Storage            | ✅         | ✅                | ✅        | ✅             |
| Knowledge/RAG              | ✅         | ✅                | ⬜        | ⬜             |
| MCP Integration            | ✅         | ✅                | ⬜        | ⬜             |

Legend: ✅ Implemented | ⬜ Planned | ❌ Not Applicable

## Debugging Tests

### Run single test with logs
```bash
RUST_LOG=debug cargo test test_name -- --nocapture
```

### Run tests with backtrace
```bash
RUST_BACKTRACE=1 cargo test
```

### Run tests in specific order
```bash
cargo test -- --test-threads=1
```

## Contributing

When adding new features:

1. Write unit tests for core functionality
2. Add integration tests for component interactions
3. Create BDD scenarios in feature files
4. Implement step definitions
5. Add property-based tests for complex invariants
6. Update this README with new test information

## Resources

- [Rust Testing Documentation](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Cucumber Rust](https://github.com/cucumber-rs/cucumber)
- [PropTest Guide](https://proptest-rs.github.io/proptest/)
- [Mockall Documentation](https://docs.rs/mockall/)
- [WireMock Documentation](https://docs.rs/wiremock/)
