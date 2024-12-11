# Simple Toy Payments Engine

## What's Not Being Done:
- Double-entry bookkeeping.
- Multiple accounts per client (e.g., `EmoneyAccount` and `DisputeAccount`).

## Safety and Robustness:
- Risk of integer overflow (simpler implementation with a small risk).

### How to Run
```bash
cargo run -- input.csv > output.csv
```

### How to Run Tests
```bash
cargo test
```

**Note:** There are integration and unit tests, but the crate is not extensively tested.

