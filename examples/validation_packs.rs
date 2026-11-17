use anyhow::{Context, Result, ensure};
use ghostline_quest_builder::{
    bootstrap::Phase,
    compiler::{self, Inputs, SceneInput},
    evidence::digest,
    manifest, native, package, quest_core,
};
use serde_json::{Value, json};
use std::{collections::BTreeSet, fs, path::Path};

fn write_json(path: &Path, value: &Value) -> Result<()> {
    fs::create_dir_all(path.parent().context("output parent")?)?;
    fs::write(path, serde_json::to_vec_pretty(value)?)?;
    Ok(())
}

fn build_pack(
    root: &Path,
    schema: &ghostline_red::schema::RedSchema,
    game: &Path,
    owner: &str,
    slot: u16,
    revision: u32,
    spot: [f64; 3],
) -> Result<Value> {
    let output = root.join(format!("{owner}-v{revision}"));
    ensure!(
        !output.exists(),
        "output already exists: {}",
        output.display()
    );
    let depot = format!(r"mod\{owner}");
    let node = |name: &str| format!("$/03_night_city/#{owner}/#{name}");
    let fact = |name: &str| format!("{owner}_{name}");
    let root_resource = format!(r"{depot}\main.questphase");
    let origin = [-2870.327293376668, -2719.007722764387, 20.475060958862];
    let quest = json!({"schema_version":1,"id":owner,"title":format!("Quest Core validation {slot}"),"stages":[{
        "id":"activity","type":"play_scene","phase_resource":format!(r"{depot}\activity.questphase"),
        "scene":format!(r"{depot}\activity.scene"),"origin":node("scene_origin"),"wait_trigger":node("approach"),
        "spawners":[node("population_npc")],"despawn_after":true,"completion_fact":fact("completed")
    }]});
    let appearance = "citizen__homeless_ma_homeless_ma_17";
    let world = json!({"name":owner,"prefab_root":format!("$/03_night_city/#{owner}"),"origin":origin,"streaming_box":"world",
        "always_loaded_sector_path":format!(r"{depot}\world.streamingsector"),"block_path":format!(r"{depot}\all.streamingblock"),
        "markers":[{"ref":"#scene_origin","sector":"always_loaded","position":origin,"scene_marker":true}],
        "triggers":[
            {"ref":"#approach","sector":"always_loaded","position":[origin[0],origin[1],origin[2]-3.0],"outline":{"width":60,"depth":60,"height":20}},
            {"ref":"#cleanup","sector":"always_loaded","position":[-2898,-2709,17.475060958862],"outline":{"width":8,"depth":8,"height":20}}
        ],
        "entities":[{"ref":"#population_npc","sector":"always_loaded","position":spot,"population_record":"Character.HomelessMan","appearance":appearance,"is_vehicle":false,"spawn_on_start":false,"always_spawned":"true_","spawn_in_view":"default__true_"}],
        "ai_spots":[{"ref":"#local_spot","sector":"always_loaded","position":spot,"yaw":26.85,"snap_to_ground":true,"workspot":r"base\workspots\common\ground\generic__stand_ground__stand_around__07.workspot"}]
    });
    let scene = json!({"name":owner,"archive_path":format!(r"{depot}\activity.scene"),"speaker_distance_interruptions":false,
        "actors":[{"key":"organiser","kind":"spawner","spawner_ref":node("population_npc"),"lipsync":false,"appearance":appearance}],
        "entry_point":{"name":"start","node":"begin"},"exit_points":[{"name":"end","node":"finished"}],
        "start_node":{"key":"begin","on_start":[{"node":"entered"}]},
        "quest_nodes":[
            {"key":"entered","kind":"fact","fact":fact("scene_entered"),"value":1,"on_out":[{"node":"revision"}]},
            {"key":"revision","kind":"fact","fact":fact("revision"),"value":revision,"on_out":[{"node":"present"}]},
            {"key":"present","kind":"wait_actor_spawned","entity_ref":node("population_npc"),"on_out":[{"node":"work"}]},
            {"key":"work","kind":"use_workspot","entity_ref":node("population_npc"),"workspot_ref":node("local_spot"),"instant":true,"finish_animation":false,"enable_idle_mode":true,"on_work_started":[{"node":"working"}]},
            {"key":"working","kind":"fact","fact":fact("work_started"),"value":1,"on_out":[{"node":"leave"}]},
            {"key":"leave","kind":"pause_condition","trigger_ref":node("cleanup"),"on_out":[{"node":"release"}]},
            {"key":"release","kind":"stop_workspot","entity_ref":node("population_npc"),"workspot_ref":node("local_spot"),"finish_animation":false,"instant":true,"on_success":[{"node":"cleaned"}]},
            {"key":"cleaned","kind":"fact","fact":fact("cleanup_requested"),"value":1,"on_out":[{"node":"finished"}]}
        ],"end_node":{"key":"finished"}
    });
    let checked = manifest::check(&quest)?;
    let manifest = checked
        .manifest
        .context(format!("invalid manifest: {:?}", checked.diagnostics))?;
    let inputs = Inputs {
        worlds: vec![world.clone()],
        scenes: vec![SceneInput {
            spec: scene.clone(),
            dialogue: json!({"spoken_lines":[],"choice_lines":[]}),
        }],
        root_resource: Some(root_resource.clone()),
        ..Default::default()
    };
    let mut build = compiler::compile(&manifest, &checked.composition, &inputs, None, false)?;
    let registration = quest_core::attach(
        &mut build,
        &quest_core::Allocation {
            slot,
            owner: owner.into(),
        },
        &Phase {
            resource: root_resource,
            input: "In1".into(),
            output: Some("Out1".into()),
        },
    )?;
    ensure!(
        registration.reserved_workspots.is_empty(),
        "test pack must not register community workspots"
    );
    let binaries = native::encode_build(&build, schema, Some(game))?;
    let expected_slot = quest_core::slot_resource(slot)?;
    ensure!(
        binaries
            .keys()
            .all(|path| path.starts_with(&format!("{depot}\\")) || *path == expected_slot),
        "pack contains resources outside its namespace and slot"
    );
    let resources = binaries.keys().cloned().collect::<BTreeSet<_>>();
    let package = package::Package {
        name: owner.into(),
        payloads: binaries,
        loose: Default::default(),
        registration: serde_json::from_slice(&build.resources[&format!("{owner}.archive.xl")])?,
        audit: json!({"purpose":"release validation only","revision":revision,"slot":slot,"quest_owned_workspots":1,"core_registrations":0}),
    };
    fs::create_dir_all(&output)?;
    let receipt = package::publish_modio(&package, &output.join("modio"))?;
    write_json(&output.join("spec/quest.json"), &quest)?;
    write_json(&output.join("spec/world.json"), &world)?;
    write_json(&output.join("spec/scene.json"), &scene)?;
    for (path, document) in &build.artifacts {
        write_json(
            &output
                .join("raw")
                .join(format!("{}.json", path.replace('\\', "/"))),
            document,
        )?;
    }
    let result = json!({"owner":owner,"slot":slot,"revision":revision,"workspot":spot,"resources":resources,"registration":registration,"package":receipt,"archive_sha256":digest(&fs::read(output.join(format!("modio/{owner}/{owner}.archive")))?),"runtime_tested":false});
    write_json(&output.join("build.json"), &result)?;
    Ok(result)
}

fn run() -> Result<()> {
    let args = std::env::args_os().skip(1).collect::<Vec<_>>();
    ensure!(args.len() == 2, "usage: validation_packs GAME_DIRECTORY OUTPUT_DIRECTORY");
    let game = Path::new(&args[0]);
    let schema = ghostline_red::schema::RedSchema::load(Some(game), None)?;
    let root = Path::new(&args[1]);
    let a = build_pack(
        root,
        &schema,
        game,
        "quest_core_validation_a",
        63,
        1,
        [-2856.327293376668, -2719.007722764387, 21.15910697937],
    )?;
    let b = build_pack(
        root,
        &schema,
        game,
        "quest_core_validation_b",
        64,
        1,
        [-2870.327293376668, -2719.007722764387, 20.475060958862],
    )?;
    let update = build_pack(
        root,
        &schema,
        game,
        "quest_core_validation_a",
        63,
        2,
        [-2864.327293376668, -2719.007722764387, 20.767471084595],
    )?;
    let paths = |pack: &Value| {
        pack["resources"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| p.as_str().unwrap().to_owned())
            .collect::<BTreeSet<_>>()
    };
    ensure!(
        paths(&a).is_disjoint(&paths(&b)),
        "independent packs overlap"
    );
    ensure!(
        paths(&a) == paths(&update),
        "update changed stable resource identities"
    );
    write_json(
        &root.join("packs.json"),
        &json!({"packs":[a,b,update],"independent_resources":true,"stable_update_resources":true,"runtime_tested":false}),
    )?;
    println!("Built and archive-verified A v1, B v1 and A v2; runtime testing remains pending.");
    Ok(())
}

fn main() -> Result<()> {
    std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(run)?
        .join()
        .map_err(|_| anyhow::anyhow!("validation compiler panicked"))?
}
