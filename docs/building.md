# Build and test the core

## Requirements

You need a Rust toolchain supporting edition 2024 and a local Cyberpunk 2077 installation. Reflection metadata is bundled in `ghostline-red` and automatically overlaid with the selected game's metadata; no schema path is required. `--schema` is an optional advanced override.

The Cargo dependencies currently use sibling checkouts:

```text
ghostline77/
  quest-core/
  quest-builder/
  ghostline-red/
```

Keep those checkouts at compatible revisions. Run the following commands from `quest-core`. Replace the example game and receipt paths with your own.

## Build

A core without additional community-workspot receipts:

```sh
cargo run --release --locked -- --game "/path/to/Cyberpunk 2077" --out builds/core
```

A core including a quest mod's registered community placements:

```sh
cargo run --release --locked -- --game "/path/to/Cyberpunk 2077" --registration "/path/to/quest-core-registration.json" --out builds/core
```

Repeat `--registration` for each supported mod's receipt. `slots.json` is the default allocation file; use `--allocations path/to/slots.json` to choose another. An allocation file declares ownership; the quest mod supplies its actual slot wrapper.

The output directory must be new. Choose another `--out` directory for each build.

## Output

| Path | Contents |
| --- | --- |
| `builds/core/modio/` | Cooked package directory and ZIP for local `r6/c77` testing. |
| `builds/core/submission/zz_ghostline_quest_core-source.zip` | Source project ZIP for mod.io cloud cooking. |
| `builds/core/submission/project/` | Unpacked upload project. |
| `builds/core/source/raw/` | Generated CR2W-JSON resources for inspection. |
| `builds/core/core-build.json` | Allocations, registration hashes and package verification results. |
| `builds/core/release-audit.json` | Exact resource inventory, hashes and vanilla overrides. |

The package directory contains:

```text
zz_ghostline_quest_core/
  zz_ghostline_quest_core.archive
  zz_ghostline_quest_core.archive.json
  zz_ghostline_quest_core.archive.xl
```

The JSON and XL files contain JSON registration data. Keep the generated package together and preserve its names. Generated output is reproducible; edit authoring inputs or generator code rather than treating these files as source.

## Submit to mod.io

Use the **source** ZIP under `submission/` with mod.io's **Cook a source file** upload option. It contains:

```text
archive/pc/mod/
  zz_ghostline_quest_core.archive
  zz_ghostline_quest_core.archive.xl
```

The `.archive.xl` file contains JSON. The local package's `.archive.json` compatibility alias is omitted from the upload project. There are no scripts or tweaks in the core, so it needs no empty script/cache/tweak directories. Reports and generated inspection files stay outside both distribution ZIPs.

The upload project and the cooked `r6/c77` layout serve different purposes. Cloud cooking is still required for a data-only upload. See CDPR's [project layout guide](https://mod.io/g/cyberpunk-2077/r/how-to-create-mods-for-cross-platform-mod-support) and [publishing guide](https://mod.io/g/cyberpunk-2077/r/how-to-publish-a-mod-on-modio).

Set the title, summary, description, version, tags and language on the mod.io listing. Use [the player description](modio-description.md) for the description and follow the linked publishing guide for listing requirements. These fields are separate from the archive-only source project.

The core has no external runtime dependencies. Dependent quest listings must explicitly name the required core version and any other requirements; do not assume the experimental dependency mechanism installs them. Approval and console testing are separate from local package verification.

## Test through the local mod.io path

Close the game before replacing installed packages. Extract the generated package ZIP into the game's `r6/c77` directory, preserving its folders. The archive should be at:

```text
Cyberpunk 2077/r6/c77/zz_ghostline_quest_core/zz_ghostline_quest_core.archive
```

For example, install the local-test ZIP from PowerShell:

```powershell
$game = 'C:/Games/Cyberpunk 2077'
Expand-Archive -LiteralPath 'builds/core/modio/zz_ghostline_quest_core.zip' -DestinationPath (Join-Path $game 'r6/c77') -Force
```

Install your quest mod's package alongside it. Avoid installing another copy of the same archive in the traditional PC mod directory, since that makes load order and conflict checks ambiguous.

Load a save and return to gameplay. Check the core's phone message, then your quest's own activation. The message is guarded by the saved `ghostline_quest_core_notified` fact, so an already-notified save should not receive it again. `ghostline_quest_core_active` records entry into the core controller.

For a release, check startup from distant locations, save/reload during the quest, completion and any replay path. Test with the required core version and without optional PC plugins supplying functionality the package claims to provide natively.

See [how startup works](how-it-works.md) to distinguish scene requests, controller activation and quest-slot completion.

The build verifies archive contents, ZIP contents and its resource-type allowlist. These checks do not run the quest or certify a console release. Test the package through the intended platform/distribution path before claiming support.

## Troubleshooting

| Symptom | Check |
| --- | --- |
| No core message | Whether this save was already notified; installed package versions; conflicting bootstrap overrides. |
| Core active, quest does not start | Check the wrapper path and `In1` input in the [integration guide](integration.md#supply-the-slot-wrapper), package priority, starting conditions and saved completion facts. |
| Empty slot keeps retrying | Whether your installed wrapper actually overrides the core's vacant wrapper. |
| NPC visible but activity does not work | Correct NodeRefs, actor ownership, spawn readiness and workspot command; for community assignments, also check registration. |
| Moved community workspot still behaves as before | Whether the core was rebuilt with the updated placement receipt. Quest-owned positions belong in the quest package. |

## Release inventory

The builder rejects resources outside the explicit core inventory: the 64 empty slot wrappers, controller, status-message resources and bootstrap resources, including the four documented vanilla overrides. Verification hashes every resource and reads back the distribution ZIPs. Development probes belong in separate test packs, not in the published core.
