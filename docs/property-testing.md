# Property-Based Testing Guide

Portly uses property-based testing with `proptest` to find edge cases that manual tests miss. Property tests run with 1000+ iterations to verify that code properties hold for a wide range of inputs.

---

## When to Use Property Tests

**Use property tests for**:
- Input validation (port numbers, PIDs, command strings)
- Parsing logic (Docker ports, framework detection)
- Serialization/deserialization (config round-trips)
- Complex algorithms (process tree traversal)
- Any code that handles user input or external data

**Use unit tests for**:
- Specific known cases and examples
- Regression tests for specific bugs
- Happy path testing
- Integration testing with specific scenarios

---

## Property Test Patterns

### Pattern 1: No Panic Testing (Fuzzing)

Test that code never panics with any input:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn prop_parse_never_panics(input in ".*") {
        // Property: Should never panic with any string input
        let result = DockerClient::parse_host_ports(&input);
        
        // Verify result is valid (empty or contains valid ports)
        assert!(result.is_empty() || result.iter().all(|&p| p > 0));
    }
}
```

**Example**: `src/docker.rs` - Fuzzes Docker port parser with random strings

### Pattern 2: Round-Trip Testing

Test that serialize → deserialize equals original:

```rust
proptest! {
    #[test]
    fn prop_config_round_trip(config in any::<Config>()) {
        // Property: Serialization should be lossless
        let toml = toml::to_string(&config).unwrap();
        let deserialized: Config = toml::from_str(&toml).unwrap();
        
        assert_eq!(config, deserialized);
    }
}
```

**Example**: `src/config.rs` - Tests config serialization round-trips

### Pattern 3: Invariant Testing

Test that certain properties always hold:

```rust
proptest! {
    #[test]
    fn prop_scanner_handles_any_port(port in 1..=65535u16, pid in 1..=100000u32) {
        // Property: Scanner should handle all valid port numbers
        let mock = MockPlatform::new().with_port(port, pid);
        let mut scanner = Scanner::new(Box::new(mock));
        
        let result = scanner.scan();
        // Should not panic, returns valid result
        assert!(result.is_ok());
    }
}
```

**Example**: `src/scanner.rs` - Tests port validation with full range

### Pattern 4: Consistency Testing

Test that same input produces same output (deterministic):

```rust
proptest! {
    #[test]
    fn prop_framework_detection_consistency(command in ".*") {
        // Property: Detection should be deterministic
        let mut detector1 = FrameworkDetector::new();
        let mut detector2 = FrameworkDetector::new();
        
        let result1 = detector1.detect(&command, None);
        let result2 = detector2.detect(&command, None);
        
        assert_eq!(result1, result2);
    }
}
```

**Example**: `src/framework.rs` - Tests framework detection consistency

---

## Running Property Tests

```bash
# Run all property tests
cargo test prop_

# Run specific property test
cargo test prop_parse_never_panics

# Run with more iterations (default is 256, we use 1000+)
PROPTEST_CASES=10000 cargo test prop_

# Run with output to see shrinking
cargo test prop_parse_never_panics -- --nocapture
```

---

## Interpreting Failures

When a property test fails, proptest automatically shrinks the input to find the minimal failing case:

```
thread 'prop_parse_never_panics' panicked at 'assertion failed'
minimal failing input: input = "0.0.0.0:99999->5000/tcp"
                                           ^^^^^
                                           Port > 65535!
```

This tells you the smallest input that causes the failure, making debugging much easier than with random fuzzing alone.

**Shrinking process**:
1. Test fails with some random input
2. Proptest tries smaller/simpler inputs
3. Finds the minimal case that still fails
4. Reports that minimal case

---

## Defining Good Properties

**Good properties**:
- ✓ Test invariants that should always hold ("never panics", "always sorted")
- ✓ Are simple and easy to understand
- ✓ Focus on edge cases and boundaries
- ✓ Test behavior, not implementation details
- ✓ Complement unit tests (don't replace them)

**Bad properties**:
- ✗ Test specific values (use unit tests instead)
- ✗ Are too complex to understand
- ✗ Duplicate existing unit tests exactly
- ✗ Don't actually test anything meaningful
- ✗ Test implementation details that may change

---

## Implementing Arbitrary for Custom Types

For round-trip testing, implement `Arbitrary` for your types:

```rust
use proptest::prelude::*;

impl Arbitrary for Config {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        (
            any::<DisplayConfig>(),
            any::<FilterConfig>(),
            any::<DefaultsConfig>(),
        )
        .prop_map(|(display, filter, defaults)| Config {
            display,
            filter,
            defaults,
        })
        .boxed()
    }
}
```

**Example**: `src/config.rs` - Arbitrary implementations for all config structs

---

## CI Integration

Property tests run automatically in CI with the rest of the test suite:

```bash
cargo test  # Runs all tests including property tests
```

To run with more iterations in CI (optional):

```yaml
- name: Run property tests with high iteration count
  run: PROPTEST_CASES=10000 cargo test prop_
```

---

## Examples in Codebase

All property tests are located in the same files as the code they test:

- **`src/docker.rs`**: Docker port parsing (fuzzing, IPv4/IPv6, multiple ports)
- **`src/framework.rs`**: Framework detection (robustness, consistency, cache)
- **`src/config.rs`**: Config serialization (round-trips, Arbitrary implementations)
- **`src/platform/mock.rs`**: Process tree traversal (depth limiting, cycle detection)
- **`src/scanner.rs`**: Scanner logic (port ranges, PID handling, sorting)

**Total**: 29 property tests across 5 modules

---

## Further Reading

- [proptest documentation](https://docs.rs/proptest/) - Official crate docs
- [proptest book](https://altsysrq.github.io/proptest-book/) - Comprehensive guide
- [Property-Based Testing Guide](https://hypothesis.works/articles/what-is-property-based-testing/) - Conceptual overview
