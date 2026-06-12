# Header Bar Repurposed as POI Ticker

> **Recorded:** 2026-06-12 12:22
> **Status:** accepted

Removed body count and credit total from the system header bar. Replaced with a live POI/anomaly ticker that shows detected anomaly badges (icon + label × count) and jumponium grade. The header now answers "what's interesting here?" instead of "how many bodies and how much money?" — body count and value are already visible in the table and inspector. This aligns with the exploration-first UX philosophy: the top bar should surface actionable intelligence, not redundant data.

## Consequences

- Header becomes a notification rail for system-level discoveries.
- Future system-wide indicators (e.g., mass code hints, Spansh status) naturally fit this bar.
- Body count/credits are not lost — they remain in the body table and inspector telemetry.
