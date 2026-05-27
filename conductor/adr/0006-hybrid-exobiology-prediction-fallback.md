# Hybrid Exobiology Prediction Fallback

> **Recorded:** 2026-05-27 17:40
> **Status:** accepted

When strict Canonn exobiology boundaries (temperature, gravity, and star type) fail due to local precision limits, rounding differences, or database drift, the exobiology panel could show "No matching species boundaries" even on planets with confirmed exobiology signals. We resolved this by extracting definitive exobiology genuses (such as "Bacterium") from the Detailed Surface Scanner `SAASignalsFound` event, and using a relaxed matching fallback (matching strictly on genus, atmosphere, and planet class) when strict filters yield no matches.
