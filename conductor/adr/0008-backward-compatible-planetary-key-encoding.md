# Backward-Compatible Planetary Key Encoding

> **Recorded:** 2026-05-27 22:05
> **Status:** accepted

## Context

We introduced the `Planetary Codex` tab under the Trip History sub-views, which aggregates scanned planets trip-wide. Beyond flat planet classes (e.g., `High Metal Content Body`, `Rocky Body`), the system needs to categorize and filter these tallies by physical and biological sub-attributes, including landability (`🚀`), terraformability (`🌍`), rings (`🪐`), and confirmed biological life (`🌿`). 

Overhauling the database schema inside the persisted `trip.json` to store a structured object per entry would break backward compatibility with existing user files, causing complete session data loss or requiring complex serialization migrations.

## Decision

We decided to keep the database model for `planetary_codex` completely unchanged as a flat `HashMap<String, u32>`. Instead of structural migrations, we encode planetary sub-attributes directly inside the Map's key strings using the composite format `PlanetClass|L|T|R|B`, where:
* `L` indicates a Landable planet.
* `T` indicates a Terraformable planet.
* `R` indicates a Ringed planet.
* `B` indicates a Biological/Life-bearing planet.

When drawing the Codex, the TUI dynamically parses the key string, splits on the `|` delimiter, and aggregates counts and status badges in real-time. 

## Consequences

* **100% Backward Compatible:** Existing flat planet keys in legacy `trip.json` files parse perfectly with zero errors or migration scripts needed.
* **Extensible & Lightweight:** New attributes can be appended dynamically in the future without schema revisions.
* **Trivial In-Memory Aggregation:** Simple string splitting is extremely fast, fully keeping TUI rendering speeds in the sub-millisecond range.
