# Jumponium System-Level Detection

> **Recorded:** 2026-06-12 12:22
> **Status:** accepted

Jumponium (FSD boost material availability) is a system-level property, not a per-body anomaly. Implemented as a separate module (`anomaly_jumponium.rs`) with its own `JumponiumResult` struct and `JumponiumGrade` enum (Basic/Standard/Premium). Stored in `App.jumponium: Option<JumponiumResult>` rather than in the per-body `App.anomalies` HashMap. Displayed as a header bar badge rather than a body table column entry. This separation reflects the semantic difference: anomalies are body-specific, jumponium is system-wide.

## Consequences

- System-level features get their own state fields in App, not shoehorned into per-body structures.
- Header bar is the natural home for system-wide indicators.
- Material source tracking (`Vec<(String, Vec<u32>)>`) enables future inspector drill-down showing which body to land on for each material.
