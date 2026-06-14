# Codex Key Single-Decrement Upgrade

> **Recorded:** 2026-06-14 08:56
> **Status:** accepted

When a body gains a new attribute (bio signals detected via `FSSBodySignals`/`SAASignalsFound`, or confirmed life via `ScanOrganic`), the planetary codex key must be upgraded (e.g., `Icy Body|L` → `Icy Body|L|B`). The original implementation used `HashMap::remove()` on the old key and inserted the **total count** into the new key — meaning one body's bio signal discovery would move ALL bodies of that class into the `|B` bucket. Fixed to decrement-by-1 / increment-by-1 across all three upgrade paths (`FSSBodySignals`, `SAASignalsFound`, `ScanOrganic`). This preserves per-body accuracy in the codex while maintaining the pipe-delimited key encoding from ADR-0008.
