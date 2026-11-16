# Connect a quest mod

## Allocate a slot

The core has 64 quest entry slots, numbered 1–64. Each belongs to one mod, identified by a stable owner ID. Coordinate the allocation with the core maintainer and record it in `slots.json` before publishing.

Add an allocation entry to the array in `slots.json`, for example:

```json
{ "slot": 2, "owner": "example_quest" }
```

The owner ID is a technical identity, not the title players see. Keep it stable between releases. Build validation rejects duplicate slots, duplicate owner IDs and invalid identifiers. Editing your own copy of `slots.json` does not reserve a slot in the published core.

## Supply the slot wrapper

Your quest mod replaces this one resource for slot 2:

```text
mod\ghostline_quest_core\slots\slot_002.questphase
```

The wrapper has this interface:

| Socket | Purpose |
| --- | --- |
| `In1` | The core starts the wrapper here. |
| `Vacant` | No quest is available; the core retries after 15 seconds. |
| `Done` | This slot has finished for the save. |

For a finite quest, connect `In1` to a call to your own root questphase, then connect that phase's completion output to `Done`. Preserve the `Vacant` output as part of the interface even if your installed wrapper never uses it.

For a persistent series controller, keep the child phase running and don't return `Done` while it still needs to listen for progression or replay requests. Returning `Done` sets the core's saved slot-completion fact; that slot will not start again on subsequent loads.

An empty slot in the core returns `Vacant`. Slots are dispatched independently: one running quest does not block the others. Your root decides when to offer its quest, using normal native conditions and triggers.

## Generate the wrapper with Quest Builder

If your compiler uses `ghostline-quest-builder`, call `quest_core::attach` after generating your quest's `Build`. The root resource must already be present in `build.artifacts`, with the input and optional completion output you declare.

```rust
use ghostline_quest_builder::{bootstrap::Phase, quest_core::{self, Allocation}};

let registration = quest_core::attach(
    &mut build,
    &Allocation {
        slot: 2,
        owner: "example_quest".into(),
    },
    &Phase {
        resource: r"mod\example_quest\main.questphase".into(),
        input: "In1".into(),
        output: Some("Done".into()),
    },
)?;

std::fs::write(
    "quest-core-registration.json",
    serde_json::to_vec_pretty(&registration)?,
)?;
```

This is a compiler integration fragment; `build` is your generated `Build`. For a persistent root, set `output: None`. If your root uses different socket names, declare those names in `Phase`; the outer slot interface remains `In1`, `Vacant`, `Done`.

### Run a complete example

[examples/slot.rs](../examples/slot.rs) loads the quest, scene and world specifications bundled in [examples/native-workspots](../examples/native-workspots), compiles all resources, attaches a slot and produces a verified local-test package. These inputs come from Quest Builder's native-workspots example; no separate checkout is required:

```sh
cargo run --release --locked --example slot -- --game "/path/to/Cyberpunk 2077" --slot 2 --out builds/world-activity
```

The output directory must be new. Use an available slot in your development installation; slot 2 here is an example, not an allocation. The example's stable owner is `world_activity`. Install its `modio/world_activity.zip` beside the core under `r6/c77`. Approach the Pacifica pavement at X=-2870, Y=-2719, Z=20.5: the NPC uses its standing workspot. Walk to X=-2898, Y=-2709 to release and despawn it. Its finite root returns `Out1`, which the wrapper translates into `Done`. Completion is saved; use a pre-test save to repeat it.

The emitted `quest-core-registration.json` has no community workspots, so this example needs no core rebuild. Change the authoring IDs, positions and gameplay for your own mod and arrange its allocation before publishing. See [build and submission layouts](building.md) before uploading; the example ZIP is for local testing.

The helper generates the wrapper, validates its root interface, removes that root's separate quest registration, and collects community-workspot registrations into the returned receipt. A slot build must not also own a standalone bootstrap.

Race Builder exposes the same choice under **Advanced: quest startup → Quest Core slot**. Series manifests use the `quest_core_slot` field.

## Package your mod

Include your allocated slot wrapper and your own gameplay resources. Install the shared core as a separate dependency; don't bundle a private core into each quest mod.

The core package is named `zz_ghostline_quest_core`. Keep your archive and package-folder names alphabetically before that name. The current loading arrangement relies on the first loaded resource winning, so your wrapper must take priority over the core's empty version.

Mods using community-registered workspots must also supply their generated registration receipt for the matching core build. Scene-local and directly assigned quest-owned workspots need no core workspot registrations. The races use the latter arrangement. See [workspot choices and examples](workspots.md).

## Saved state

The generated helper uses these facts, where `owner` is your allocation's owner ID:

| Fact | Meaning |
| --- | --- |
| `owner_core_entered` | The wrapper has called the quest root. |
| `owner_core_completed` | The finite root has completed. |
| `owner_core_disabled` | On wrapper entry, a positive value makes it return `Vacant`. |
| `ghostline_quest_core_slot_002_done` | The core has received `Done` from slot 2. |

These are technical facts, not player-facing settings. Your quest still owns its own progress, rewards and replay logic. A completion guard does not replace guarding individual one-time rewards.

## Startup and conflicts

The shared core owns the interesting-conversation area, vendor context, registered
startup scene and persistent quest-host listener. Quest mods attached to slots
do not generate another global bootstrap.

See [how the bootstrap works](how-it-works.md) for the request flow, saved guards
and the four vanilla resources it overrides. Another mod replacing those
resources can conflict even when its quest slot is different.
