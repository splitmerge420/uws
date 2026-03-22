# Benchmarks

Add Criterion benchmarks here. The `.github/workflows/bench.yml` workflow will automatically
run them on every push to `main` and track results in the `gh-pages` branch.

## Example

```rust
// benches/my_bench.rs
use criterion::{criterion_group, criterion_main, Criterion};

fn my_bench(c: &mut Criterion) {
    c.bench_function("my_function", |b| {
        b.iter(|| {
            // your code here
        });
    });
}

criterion_group!(benches, my_bench);
criterion_main!(benches);
```

Then add to `Cargo.toml`:

```toml
[[bench]]
name = "my_bench"
harness = false

[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }
```
