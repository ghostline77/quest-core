//! Complete slot integration using Quest Builder's authored world-activity example.
use anyhow::{Context, Result, ensure};
use clap::Parser;
use ghostline_quest_builder::{
    bootstrap::Phase,
    compiler::{self, Inputs, SceneInput},
    manifest, native, package, quest_core,
};
use serde_json::{Value, json};
use std::{fs, path::PathBuf};

#[derive(Parser)]
struct Args {
    #[arg(long)]
    game: PathBuf,
    #[arg(long)]
    schema: Option<PathBuf>,
    #[arg(long)]
    slot: u16,
    #[arg(long)]
    out: PathBuf,
}

fn build(args: Args) -> Result<()> {
    ensure!(!args.out.exists(), "output directory must be new");
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../quest-builder/examples/native-workspots");
    let load = |name: &str| -> Result<Value> {
        Ok(serde_json::from_slice(&fs::read(source.join(name))?)?)
    };
    let checked = manifest::check(&load("quest.json")?)?;
    let manifest = checked.manifest.context("invalid example quest manifest")?;
    let root = format!(r"mod\{}\main.questphase", manifest.id);
    let inputs = Inputs {
        worlds: vec![load("world.json")?],
        scenes: vec![SceneInput {
            spec: load("scene.json")?,
            dialogue: load("dialogue.json")?,
        }],
        root_resource: Some(root.clone()),
        ..Default::default()
    };
    let mut build = compiler::compile(&manifest, &checked.composition, &inputs, None, false)?;
    let registration = quest_core::attach(
        &mut build,
        &quest_core::Allocation {
            slot: args.slot,
            owner: manifest.id.clone(),
        },
        &Phase {
            resource: root,
            input: "In1".into(),
            output: Some("Out1".into()),
        },
    )?;
    let schema = ghostline_red::schema::RedSchema::load(Some(&args.game), args.schema.as_deref())?;
    let binaries = native::encode_build(&build, &schema, Some(&args.game))?;
    let package = package::Package {
        name: manifest.id.clone(),
        payloads: binaries,
        loose: Default::default(),
        registration: serde_json::from_slice(
            &build.resources[&format!("{}.archive.xl", manifest.id)],
        )?,
        audit: json!({"example":"quest-owned world workspot","slot":args.slot}),
    };
    fs::create_dir_all(&args.out)?;
    package::publish_modio(&package, &args.out.join("modio"))?;
    fs::write(
        args.out.join("quest-core-registration.json"),
        serde_json::to_vec_pretty(&registration)?,
    )?;
    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();
    std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(|| build(args))?
        .join()
        .map_err(|_| anyhow::anyhow!("example compiler panicked"))?
}
