# Self-contained value calculation

> **Recorded:** 2026-05-27 00:10
> **Status:** accepted

Body exploration values are calculated locally using community-derived formulas ported from Pioneer's `body_calc.py` (~100 lines of Python → Rust). No EDSM API call is needed for base value estimates.

The formulas are well-documented and stable — they haven't changed since Frontier last adjusted exploration payouts. They account for body type, mass, terraformability, first discovery bonus, and first mapping bonus. Porting to Rust is trivial and gives us instant, offline value calculation with no network latency.

## Consequences

- Must port and test the value formulas against known correct outputs
- Values may drift if Frontier adjusts payout formulas in a future game update (unlikely but possible — would require a code update)
- No API rate limits or auth tokens needed for base values
