# Simple Toy Payments Engine

## What's Not Being Done:
- Double-entry bookkeeping
- Multiple accounts per client (e.g., `EmoneyAccount` and `DisputeAccount`)
- No duplicate transactions handling
- No Fine-Grained Locking with Internal Mutexes

## Safety and Robustness:
- Risk of integer overflow (implementation simplicity vs risk)

### How to Run
```bash
cargo run -- input.csv > output.csv
```

### How to Run Tests
```bash
cargo test
```

**Note:** There are integration and unit tests, but the crate is not extensively tested

**Note:** When running streams concurrently, consider different chronological requirements (file-scope vs inter-file) 
