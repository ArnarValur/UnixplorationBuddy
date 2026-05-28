# Domain Glossary

> Last refined: 2026-05-27T14:05

## Entities

| Term | Definition |
|------|-----------|
| **System** | A star system — the top-level container. Identified by name from journal `FSDJump` events. Has a completion state (all bodies discovered vs incomplete). |
| **Body** | A celestial object within a system: star, planet, moon, or belt cluster. Uniquely identified by `BodyID` within a system. |
| **Body Hierarchy** | Parent→child tree derived from Elite's naming convention (e.g., "A 1 a" is a moon of planet "A 1" orbiting star "A"). Determines display ordering. |
| **Scan State** | Discovery progression of a body: unseen → honked (system-level discovery scan) → FSS scanned (detailed) → DSS mapped (surface). Each state unlocks more data. |
| **Body Value** | Credit value of exploration data for a given body. Calculated from body mass, type, terraformability, and modifiers for first discovery / first mapping. Self-contained formulas ported from community-derived constants. |
| **Bio Signal** | Biological life detected on a body's surface. Journal events provide an integer count of signals. |
| **Geo Signal** | Geological features (volcanism, geysers, fumaroles) detected on a body. Integer count from journal events. |
| **Species Prediction** | Filtering Canonn's dataset by body conditions (atmosphere, body type, gravity, temperature, star class) to predict which biological species may be present on a body. Run only when bio signals count > 0. |
| **Vista Genomics Price** | Credit value for selling analysed biological data at Vista Genomics stations. Per-species, sourced from Canonn's dataset. |
| **Journal Event** | A JSON record written by Elite Dangerous to the player journal file (`Journal.YYYY-MM-DDThhmmss.log`). Primary data source for all game state. |
| **Trip** | A contiguous exploration session, tracked for accumulated value — systems visited, bodies discovered, bio analysed, total credits earned. |
| **Status.json** | Real-time cockpit status file written by the game client, watched by the application to sync targeted destinations and ship conditions. |
| **Target Sync** | Dynamic TUI mechanism that parses the targeted destination from `Status.json` and automatically highlights and displays the inspector telemetry for that body. |
| **Trip Codex** | Explorer's registry breakdown under the Trip tab, housing the stellar class log, planet class log, and organic species log. |
| **ScanOrganic** | In-game journal event indicating exobiology genetic sampler stages (Log, Analyse) used to tally completed exobiology codex entries. |
| **NavRoute.json** | In-game plotted navigation route file written by the client, watched to populate the Route Exploration view. |
| **Route Exploration** | Plotted navigation pipeline tracking jump classes, fuel-scoop indicators, EDSM discovery states, EDSM system values, and CMDR first-mappers. |

## Relationships

| Subject | Verb | Object | Notes |
|---------|------|--------|-------|
| System | contains | Body | 1:N — a system has zero to many bodies |
| Body | has parent | Body | Self-referential — moons orbit planets, planets orbit stars |
| Body | has | Scan State | 1:1 — each body has exactly one current scan state |
| Body | has | Bio Signal | 0:N — a body may have zero or many bio signals |
| Body | has | Body Value | 1:1 — computed from body properties and scan state |
| Species Prediction | applies to | Body | N:M — predictions are filtered per body conditions |
| Trip | aggregates | System | 1:N — a trip spans multiple systems visited |
| Target Sync | auto-focuses | Body | Real-time cockpit destination tracking |
| Trip Codex | aggregates | Star / Planet / Species | Tallies individual categories encountered on a trip |
| Route Exploration | tracks | System | Dynamic list of upcoming waypoints from NavRoute |

## Terminology Boundaries

| Term | Is NOT | Clarification |
|------|--------|---------------|
| Body Value | Vista Genomics Price | Body Value = exploration scan credits. VG Price = bio sell credits. Different reward systems. |
| Honk | FSS Scan | Honk (Discovery Scan) reveals body count. FSS resolves individual bodies. Different scan stages. |
| Bio Signal | Species Prediction | Signal = in-game detection count. Prediction = Canonn data match. Signal confirms life exists; prediction tells you what it might be. |
| Target Sync | Selection Navigation | Target Sync = automatic cockpit focus tracking. Selection Navigation = manual TUI list selection. |
