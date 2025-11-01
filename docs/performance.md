# Performance Benchmarking Playbook

The workbench performance section instruments two synthetic scenarios: a virtualized
data grid and a 200k-line editor buffer. These rigs provide fast, repeatable signals
without depending on external services.

## Methodology

1. **Virtualized list** – Each run drives `data::VirtualListBenchmark` with the active
   row count, viewport height, and overscan values. Synthetic scroll samples update
   FPS, render latency, and memory charts once per frame.
2. **Editor stress test** – The editor view fabricates a 200,000-line Rust buffer and
   samples keystroke-to-paint latency, LSP update lag, and heap growth as options
   (syntax highlighting, LSP) toggle on and off.
3. **Run persistence** – Every benchmark suite execution serializes a
   `BenchmarkRunRecord` through `platform::ConfigStore`, enabling longitudinal
   comparisons.

## Recommended practices

- Prefer constant row heights for lists. It keeps virtualization math predictable and
  makes overscan buffers easier to tune.
- Lazy-load expensive cell content. Render a cheap shell in the viewport and hydrate
  detail views asynchronously once they appear on screen.
- Batch editor updates before notifying the language server. Large buffers benefit
  from debouncing to avoid compounding LSP latency.
- Capture baseline runs whenever component settings change. The history charts surface
  regressions immediately.

## Known limitations

- Metrics are synthesized in-process. Use them to spot trends and validate knobs, then
  confirm with production telemetry.
- GPU timelines are approximated; attach an external tracer such as tracy or wgpu
  captures for hardware-level investigations.
- The editor preview truncates after a few hundred lines so the UI remains responsive
  even on machines with constrained VRAM.
