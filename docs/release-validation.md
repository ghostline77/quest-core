# Quest Core release validation

The candidate built from `ab09b6c` passed native resource readback, the 75-resource inventory, four vanilla override checks, and both ZIP layout checks. The SHA-256 of the source upload is `889d930b0da19fc984479e2265c01439d7673eb0e6296371cbc1259b2ae6ddb6`. The local-test ZIP has SHA-256 `1e9ba51b39d93b3d47a225762324e780c6930124db6bbc9e06ce7397e1c8e110`. Generate both with the [build instructions](building.md).

Earlier gameplay checks confirmed that the activation message appears once, stays suppressed after save/reload and fast travel, and that the race series works with the core. They did not cover the entire release matrix.

| Pending test | Pass condition |
| --- | --- |
| Core alone, fresh and existing-progress saves in Watson, Pacifica and the Badlands | Message arrives once in normal gameplay, empty slots stay idle, reload is clean. Use separate manual saves. |
| Two independent test packs on distinct slots | Both start, NPC and workspot behavior survives reversing install order, each pack cleans up its own activity. |
| Update one pack and retain the same core and second pack | Updated placements load, the other pack still works, and its archive and the core remain byte-identical. |
| Core update during an active quest and after completion | Progress, rewards, completion and replay behavior remain correct across reload. |
| Remove and reinstall one pack | Record the effect on saves made before startup, during play and after completion. Document the observed restrictions. |
| Final PC candidate through the local mod.io path | Repeat startup and dependent-pack checks with optional PC plugins and development helpers disabled. |
| mod.io and console | Supply original listing art, submit the source ZIP, check the cloud cook and approved package, then test on PS5 alone and with a dependent pack. |

Record the game version, package hashes, install order, save, location and observed outcome for each run. Keep the local-test ZIP and source submission separate.
