# Asynchronous EDSM Enrichment via Background Worker and ureq

> **Recorded:** 2026-05-27 14:05
> **Status:** accepted

## Context

To provide rich route diagnostics (estimated total values, valuable body counts, and historical discoverers) on the Route Exploration view, we must fetch data from the Elite Dangerous Star Map (EDSM) API. 

Making live network requests on the main terminal user interface (TUI) thread would block Crossterm's event processing loop and freeze the rendering frame rates, resulting in a laggy and unresponsive experience for the pilot.

## Decision

We will implement an asynchronous enrichment pipeline by spawning a dedicated, lightweight background worker thread that performs the network requests and streams results back to the main loop via a standard message-passing channel (`std::sync::mpsc`).

Furthermore, we will use the synchronous **`ureq`** HTTP library for these background requests instead of introducing a heavy asynchronous runtime like `tokio`.

## Consequences & Trade-offs

### Pros
* **Responsive UI:** The TUI's rendering and keyboard navigation remain sub-millisecond fast, completely unaffected by latency, rate-limiting, or connection dropouts from the EDSM API.
* **Dependency Lightness:** `ureq` (coupled with standard OS threads) is exceptionally lightweight and compiles in seconds, unlike the extensive dependency tree required by `tokio` and `reqwest`.
* **Code Simplicity:** Avoids transforming the entire TUI codebase into complex `async fn` structures, keeping the Rust code clean and linear.

### Cons & Trade-offs
* **System Threads:** Uses native OS threads rather than lightweight green tasks, but because we only need a single dedicated background thread to handle EDSM queries, the runtime overhead is virtually zero.
* **API Rate-Limiting:** The worker thread must implement internal throttling (or debouncing) to respect EDSM's API guidelines and prevent rate-limiting bans.
