# Workspots

A workspot describes an activity and its placement, such as sitting in a chair, standing at a stall or leaning against a wall. Choose how the activity is assigned to the NPC. Whether the NPC belongs to a quest, appears in a scene or stays in the world does not by itself determine which kind you need.

## Workspot types

| Kind | Use it when | Example | Core workspot registration? |
| --- | --- | --- | --- |
| **Scene-local workspot** | The scene defines the animation placement and controls the activity's lifetime. | A character sits in a chair during a conversation or performs a scripted animation. | No. |
| **Quest-owned world workspot** | Your quest places a spot and explicitly tells an NPC to use it. A quest phase or scene can issue that command. | Spectators standing beside a race grid, or a quest NPC leaning against a wall until the player approaches. | No. |
| **Community-registered workspot** | You use the native community system to assign NPCs to registered world activities. | A resident's schedule assigns a chair in the morning and a stall later, or community NPCs select from available activity spots. | Yes. |

Community registration applies when the native community system assigns the activity. With explicit scene or quest control, your mod manages spawning, activity selection, timing and cleanup instead. An NPC's presence in the world or participation in dialogue does not by itself require community registration.

### NPC ownership and scene participation

NPC ownership and workspot placement are separate choices. A population spawner can own an NPC while a scene temporarily acquires it for dialogue. That does not make its placed world workspot scene-local. Likewise, a community NPC can participate in quest scenes without changing how its community activities are registered.

A vendor can use either arrangement. A community-managed vendor needs registration for the stall activity assigned by that community. A quest can instead spawn a vendor and explicitly send it to a quest-owned stall workspot. Both can use a scene for dialogue and opening the shop. Being a vendor, being global or having dialogue does not inherently require community registration.

## Quest-owned world activities

The [Quest Builder example](https://github.com/ghostline77/quest-builder/tree/main/examples/native-workspots) contains ordinary quest, scene and world specifications. The `play_scene` stage activates the population spawner, waits for the NPC, plays the scene, then optionally deactivates the spawner after the scene releases its actor. The scene starts and stops the workspot through direct NodeRefs; no community entry name is needed.

Keep the spawner and spot streamed while they are needed. Author each position at the actual ground height: moving a spot across a slope requires updating Z as well as X and Y. Stop the activity before despawning the NPC.

This direct assignment does not use the core's workspot bank. It also does not supply community schedules or update an existing community's registered placements.

## Community registration

This section applies only when using community-registered workspots. A quest entry slot starts your controller; its workspot bank reserves identities for community activities. Using a quest entry slot does not require using its workspot bank.

### Reserve identities in your bank

Every quest slot has 2,048 workspot IDs, numbered 0–2,047. Use the same quest slot number that your mod owns. For example, slot 2, workspot 0 has this reference:

```text
$/03_night_city/#ghostline_quest_core/workspots/slot_002/#spot_0000
```

In a Quest Builder integration, generate it with:

```rust
let reference = ghostline_quest_builder::quest_core::workspot_reference(2, 0)?;
```

Use the reference as the world spot's `ref`. Give each distinct workspot its own index and keep those assignments stable between releases. Don't use another mod's bank. Exceeding the bank requires an allocation/design change; it does not make the next index valid automatically.

### Register the placements

1. Generate your communities and their actual world workspot nodes in your quest mod.
2. Attach the quest to its core slot. The helper extracts the persistent registrations into `quest-core-registration.json` and records which reserved indices your mod uses.
3. Supply that receipt to the core maintainer. Build the shared core with `--registration path/to/quest-core-registration.json`.
4. Distribute and test your quest with the matching core version.

The core initially registers all reserved IDs. Used records take their position and rotation from the supplied receipts; unused records retain placeholder positions. The quest mod still supplies its own world nodes, communities and gameplay.

### Why a core update is currently required

A registered workspot has both a placed world node in the quest mod and a separate startup registry record containing its identity, position and rotation. Replacing the world node alone does not update that record. Reserving an ID with placeholder coordinates does not let the quest independently relocate the registered placement.

The core includes these records in `worldCommunityRegistryNode.workspotsPersistentData`, an inline array in the vanilla sector it overrides:

```text
base\worlds\03_night_city\_compiled\default\always_loaded_1.streamingsector
```

This is a shared vanilla sector containing community definitions, spawn phases, activity registrations, crowd creation data, world markers, patrol setup and persistent traffic and distant-lighting nodes. The core preserves the surrounding vanilla content, adjusts the bootstrap vendor's spawn setup and includes the additional workspot registrations in the initial registry.

Independent quest-side registration is not currently supported safely without the PC ArchiveXL plugin:

- Replacing the entire sector from a quest archive also replaces the shared registrations belonging to other mods.
- Supplying another sector with an additional registry batch uses an unsafe native merge path. Desktop [ArchiveXL supplies a native merge fix](https://github.com/psiberx/cp2077-archive-xl/blob/main/src/App/Extensions/WorldStreaming/Extension.cpp#L733); cross-platform packages cannot depend on that PC DLL.
- The inline registry records are not references to separate per-slot position resources that an add-on can replace independently.

The supported arrangement is therefore to rebuild the shared core with the quest's updated receipt when adding or moving community-registered workspots. This is a limitation of the current integration, not proof that no other native approach could exist.

The bootstrap vendor uses a community-registered workspot to supply the NPC and activity context for the core's conversation scene. See [the bootstrap flow](how-it-works.md#vendor-context-and-streaming). Other vendors and quest starters can use explicitly assigned activities instead.

For a core release supporting several quest mods, include every supported mod's current receipt, even if only some of those mods are installed during your test. The core is a shared registry, not a build tailored to one player's installed set.

The builder rejects duplicate reserved indices, use of another slot's bank, conflicting unreserved IDs and unsupported receipt versions. Unreserved community workspots are also supported through receipts, but require the same shared-core rebuild process and have no reserved-bank ownership protection.

## Core updates

| Change | Workspot registry rebuild? |
| --- | --- |
| Change dialogue, quest conditions or rewards | No, if registered placements stay unchanged. |
| Edit a route or visual effect | No, unless it also changes registered placements. |
| Add or move a community-registered workspot | Yes, with an updated registration receipt. |
| Add or move a quest-owned world spot used directly by a population NPC | No. |
| Change only a scene-owned animation workspot | No persistent registry update. |

Compare the generated registration receipt when deciding whether a content change needs a matching core release. Registration occurs during startup; a late quest-side batch is not a replacement for the core's initial registration.
