# Data Crate

Utility data factories used across demos and benchmarking helpers for virtual
lists and tables.

## Highlights

- `generate_transactions(count)` builds deterministic financial transactions that
  are suitable for seeding list/table components.
- `export_transactions(count)` serializes the generated data into JSON for mock
  API responses.
- `VirtualListBenchmark` estimates buffer sizes for virtualized GPUI lists.
- When the `async-loaders` feature is enabled the crate exposes
  `load_transactions_async`, a Tokio powered helper that mimics network latency.
