# Ghostline Quest Core

Ghostline Quest Core starts Cyberpunk 2077 quest mods through a shared native
bootstrap. It provides 64 independent quest entry slots and sends a
**Ghostline Quest Core active** phone message once per save.

Each quest mod supplies its own story, characters, conditions and progression.
The core does not require a visit to V's H10 apartment.

This repository documents integration for modders. The
[mod.io description](docs/modio-description.md) is written for players.

| Task | Documentation |
| --- | --- |
| Understand startup and the interesting-conversation area | [How it works](docs/how-it-works.md) |
| Connect a quest to a slot | [Integration guide](docs/integration.md) |
| Choose scene, quest or community workspots | [Workspot guide](docs/workspots.md) |
| Build, package and test the core | [Build instructions](docs/building.md) |
| Check resource conflicts | [Vanilla overrides](docs/how-it-works.md#vanilla-overrides) |

## How it works

An interesting-conversation area uses an existing vendor's NPC and workspot
context to start a registered scene. The scene sets a request fact. A listener
in the game's active persistent quest graph receives that request and launches
the core controller.

The core's area and vendor streaming changes allow this to happen away from the
vendor. Players do not have to visit the stall or select a conversation option.
The controller starts each installed quest through its allocated slot.

See [the startup flow](docs/how-it-works.md) for the scene registration, vendor
changes, saved guards and overridden resources.

## Quest slots

A quest mod replaces one slot wrapper, such as
`mod\ghostline_quest_core\slots\slot_002.questphase`. Its interface is
`In1`, `Vacant` and `Done`. The wrapper calls the mod's own root questphase;
the shared core is installed separately.

Coordinate a unique allocation in [slots.json](slots.json) before publishing.
Follow the [integration guide](docs/integration.md) to generate and package the wrapper.

## Workspots

Scene-local and directly assigned quest-owned workspots need no core registry
update. Community-registered workspots use a shared startup registry; adding or
moving those placements currently requires a matching core build.

The race series uses population-owned NPCs and quest-owned world workspots,
with no community workspot registrations. See
[workspot choices and examples](docs/workspots.md).

## Compatibility

The core owns four [vanilla overrides](docs/how-it-works.md#vanilla-overrides).
Separate quest slots do not merge conflicts with other mods replacing those resources.

Quest progress and completion facts are saved. Rebuilding an archive does not
reset that progress. Test graph updates and removal with separate saves.

The packager checks its cross-platform resource allowlist. Local build checks
do not establish PS5 compatibility; validation through mod.io and on the target
platform remains separate.
