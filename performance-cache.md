# Performance & Caching Notes

> WHY: Document current perf characteristics & low-risk improvement knobs. 中文：描述現有快取與潛在優化點。

## 1. Existing Caches

| Layer                  | Mechanism                          | Scope        | Hit Path                           | Miss Cost                  |
| ---------------------- | ---------------------------------- | ------------ | ---------------------------------- | -------------------------- |
| Config                 | `OnceCell<RwLock<Cofg>`            | Global       | Almost every request (Cofg::new)   | Disk read + deserialize    |
| Template Engine        | `OnceCell<RwLock<TemplateEngine>>` | Global       | Each markdown render               | Engine construction        |
| Request Derived Values | `HttpRequest.extensions`           | Per-request  | Multiple handler branches + logger | Recompute path decode/join |
| Bytecode Cache         | mystical_runic internal flag       | Engine-level | Template compile                   | Parse + compile template   |

## 2. Hot Reload Costs

| Feature                           | Extra Overhead When Enabled   | Rationale                      |
| --------------------------------- | ----------------------------- | ------------------------------ |
| Config force reload (`get(true)`) | One lock write + disk read    | Developer explicit action only |
| Engine rebuild per render         | Lock write + new engine alloc | Immediate template reflection  |

中文：hot_reload 僅在開發期啟用，成本有限且局限在模板與設定重建。

## 3. Potential Bottlenecks

| Area                          | Symptom                               | Profiling Signal                   |
| ----------------------------- | ------------------------------------- | ---------------------------------- |
| Large markdown parse          | High CPU time in `markdown_ppp`       | Flamegraph heavy parse stack       |
| Frequent identical md renders | Repeated parse & render for same file | High count of identical file reads |
| Huge public tree TOC          | Slow `/` when index absent            | Long glob walk time                |

## 4. Improvement Options (Incremental)

| Option                                          | Type             | Effort | Risk                            | Expected Gain                             |
| ----------------------------------------------- | ---------------- | ------ | ------------------------------- | ----------------------------------------- |
| HTML output cache keyed by (abs path, mtime)    | Runtime feature  | Medium | Need invalidation correctness   | Avoid repeat parse for popular docs       |
| Async pre-warm cache on startup (scan recent N) | Optional feature | Medium | Startup delay                   | Faster first hits                         |
| TOC memoize with directory mtime hash           | Runtime feature  | Low    | Stale if partial changes missed | Faster `/` under big trees                |
| Config validation pass (log warnings)           | Startup          | Low    | Minimal                         | Early detection of misconfig              |
| Rate limit engine rebuild logging               | Dev UX           | Low    | None                            | Cleaner logs when saving template rapidly |

## 5. Suggested Roadmap

1. Add simple in-memory LRU (e.g. `hashbrown + linked-hash`) for rendered pages
2. Add quick `public_path` traversal to find largest / most popular candidate files (heuristic: size)
3. Expose feature flags (Cargo `--features perf-cache`) to keep base binary minimal

## 6. Edge Considerations

| Edge                           | Handling Today           | Note                            |
| ------------------------------ | ------------------------ | ------------------------------- |
| File deleted mid-request       | Read error → 500         | Acceptable; infrequent race     |
| Large single file (> a few MB) | Fully buffered           | Could stream in future          |
| Path traversal (`..`)          | Not explicitly sanitized | Consider canonical prefix check |

## 7. Micro Benchmarks (Hypothetical Setup)

To evaluate improvements, measure:

- Cold vs warm md2html (same file) time (ns/op)
- Throughput under concurrency (wrk / bombardier) with mixed static + md
- Latency added by hot_reload vs disabled

## 8. Non-Goals (Currently)

| Item                                 | Reason                                        |
| ------------------------------------ | --------------------------------------------- |
| Persistent cache across restarts     | Complexity > benefit for lightweight use case |
| Distributed cache                    | Out of scope; single-node focus               |
| Partial incremental markdown parsing | Requires upstream parser support              |

## 9. Summary

Current design intentionally minimal: rely on OS page cache + lightweight global objects. Only implement additional caching once real workloads demonstrate need.
