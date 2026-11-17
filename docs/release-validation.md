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
| mod.io and console | Use the [listing worksheet](modio-listing.json) and [original gameplay cover](modio-cover.png), submit the source ZIP, check the cloud cook and approved package, then test on PS5 alone and with a dependent pack. |

Record the game version, package hashes, install order, save, location and observed outcome for each run. Keep the local-test ZIP and source submission separate.

## Reproduce the two-pack test

Generate A v1, B v1 and A v2 with their slot wrappers and archive receipts from the repository root. Use a new output directory and a test profile with slots 63 and 64 available.

```powershell
cargo run --release --locked --example validation_packs -- $env:GAME_ROOT work/validation-packs
```

Start each row from a fresh copy of the same uninitialized save; do not reuse a save whose slot has completed. Preserve the package names in every row.

| Run | Extraction/installation sequence |
| --- | --- |
| 2a | Core, A v1, B v1. |
| 2b | B v1, A v1, core last. |
| 2c | A v1, core, B v1. |

Cold-launch each row. At a distant spawn, both `quest_core_validation_a_core_entered` and `quest_core_validation_b_core_entered` must be 1. Each loaded slot graph must contain its correct owner, not the empty wrapper. `ghostline_quest_core_slot_063_done` and `_064_done` should still be 0 while the activities wait.

Travel near X=-2870.3273, Y=-2719.0077, Z=20.4751 (Pacifica pavement). Both approach triggers cover this area. Before entering the cleanup trigger, both NPCs must be visible and using their separate standing spots. Read these facts for each owner:

```text
quest_core_validation_a_scene_entered = 1
quest_core_validation_a_work_started = 1
quest_core_validation_a_revision = 1
quest_core_validation_b_scene_entered = 1
quest_core_validation_b_work_started = 1
quest_core_validation_b_revision = 1
```

Verify native workspot occupancy as well as facts. Walk to X=-2898, Y=-2709 to release the activities. Both `_cleanup_requested` and `_completed` facts must become 1; both NPCs must despawn. The wrappers connect their finite roots' `Out1` output to `Done`: both owner `_core_completed` facts and core slot 063/064 completion facts must then become 1. This also checks independent cleanup and saved completion guards.

Installation sequence is not assumed to determine resource priority. The important evidence is that all supported sequences load populated wrappers ahead of the core's vacant versions with the distributed names. Also compare the package's hashes across rows. A deliberately renamed archive sorting after `zz_...` is an optional negative control, not a supported configuration.

### Check a placement update

1. Record the installed core hash; use A v1 and B v1 to confirm both spots as above.
2. Quit without saving over the baseline. Replace only A with A v2, leaving the core and B byte-identical.
3. Cold-load an uninitialized save and approach the same pavement.
4. A must use X=-2864.327293376668, Y=-2719.007722764387, Z=20.767471084595 and set `_revision=2`. A v1 used X=-2856.327293376668, same Y, Z=21.15910697937. B must remain at X=-2870.327293376668, same Y, Z=20.475060958862 with `_revision=1`.
5. Confirm A and B occupancy and cleanup, empty registration receipts and an unchanged core hash.

Do not use an already-running saved activity to judge a not-yet-started placement update: saved native graph/actor state is a separate update case.
