# Offline Exobiology Species Prediction matching Canonn Boundaries

> **Recorded:** 2026-05-27 14:05
> **Status:** accepted

## Context

Exobiologists operating in the deep black require immediate species prediction, exobiology Vista Genomics payouts, and biological sample counts without internet access. Live web lookups are impossible during long exploration journeys far from stations.

We have gathered 39 markdown and JSON files containing exobiology distribution data compiled by Canonn Research (`conductor/canonn-data/`). We need a reliable mechanism to process and evaluate this exobiology data boundaries inside our Rust TUI app.

## Decision

We will compile and embed the Canonn Research exobiology species boundaries directly into the UnixplorationBuddy binary at build-time.

The application will run an **offline prediction engine** that parses physical conditions (atmosphere type, body class, surface gravity, surface temperature, primary star class) from FSS journal `Scan` events in real-time and evaluates them against the embedded dataset boundaries.

To avoid visual noise and redundant computations, the exobiology prediction panel will only trigger and render **if and only if** the in-game scanner reports a biological signal count > 0 (via `FSSBodySignals` or `SAASignalsFound` events).

## Consequences & Trade-offs

### Pros
* **Offline-First:** Core exobiology predictions, species boundary checking, and credit valuations work perfectly without an internet connection, satisfying a primary differentiator of UnixplorationBuddy.
* **Instantaneous Matching:** Deserialized static lookups run in sub-millisecond speeds, avoiding API query overheads.
* **Focused Telemetry:** Hiding the exobiology predictions pane for planets without biological signals ensures high visual density is only applied to relevant systems.

### Cons & Trade-offs
* **Static Dataset:** Embedding boundaries means the dataset is fixed at compile-time and can drift if exobiology parameters are altered in game updates, requiring a new package release to update. However, exobiology parameter boundaries are exceptionally stable.
* **Binary Size:** Bundling exobiology parameters adds a minor footprint to the compiled binary, but because the compressed parameters are extremely compact, the final binary remains well under 10MB.
