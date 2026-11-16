use anyhow::{Context, Result, ensure};
use clap::Parser;
use ghostline_quest_builder::{native, package, quest_core};
use serde_json::json;
use std::{fs, path::PathBuf};
mod release;

#[derive(Parser)]
#[command(version, about = "Build the shared native Ghostline Quest Core")]
struct Args {
    #[arg(long)]
    game: PathBuf,
    #[arg(long)]
    schema: Option<PathBuf>,
    #[arg(long, default_value = "slots.json")]
    allocations: PathBuf,
    /// Registration receipts from supported quest-pack builds, including absent packs.
    #[arg(long)]
    registration: Vec<PathBuf>,
    #[arg(long)]
    out: PathBuf,
}

fn build(args: Args) -> Result<()> {
    ensure!(!args.out.exists(), "output must be a new directory");
    let allocations: Vec<quest_core::Allocation> =
        serde_json::from_slice(&fs::read(&args.allocations)?)?;
    quest_core::validate_allocations(&allocations)?;
    let registrations = args
        .registration
        .iter()
        .map(|path| -> Result<quest_core::Registration> {
            serde_json::from_slice(&fs::read(path)?)
                .with_context(|| format!("registration {}", path.display()))
        })
        .collect::<Result<Vec<_>>>()?;
    for registration in &registrations {
        ensure!(
            allocations
                .iter()
                .any(|a| a.slot == registration.allocation.slot
                    && a.owner == registration.allocation.owner),
            "unallocated quest pack {} in slot {}",
            registration.allocation.owner,
            registration.allocation.slot
        );
    }
    let build = quest_core::documents(&registrations)?;
    let schema = ghostline_red::schema::RedSchema::load(Some(&args.game), args.schema.as_deref())?;
    let binaries = native::encode_build(&build, &schema, Some(&args.game))?;
    let inventory = release::audit(&binaries)?;
    let name = quest_core::ID;
    let registration = serde_json::from_slice(
        build
            .resources
            .get(&format!("{name}.archive.xl"))
            .context("core registration")?,
    )?;
    let workspots = build
        .bootstrap
        .as_ref()
        .context("core bootstrap")?
        .workspot_count();
    let audit = json!({"format":"ghostline-quest-core-build","schema_version":1,"slot_interface_version":1,"slots":quest_core::SLOT_COUNT,"allocations":allocations,"registered_workspots":workspots,"runtime_verified":false,"registration_sha256":args.registration.iter().map(|p|Ok((p.display().to_string(),ghostline_quest_builder::evidence::digest(&fs::read(p)?)))).collect::<Result<std::collections::BTreeMap<_,_>>>()?});
    let package = package::Package {
        name: format!("zz_{name}"),
        payloads: binaries,
        loose: Default::default(),
        registration,
        audit: audit.clone(),
    };
    let parent = args.out.parent().context("output parent")?;
    fs::create_dir_all(parent)?;
    let stage = tempfile::Builder::new()
        .prefix(".quest-core-")
        .tempdir_in(parent)?;
    let receipt = package::publish_modio(&package, &stage.path().join("modio"))?;
    for (path, document) in &build.artifacts {
        let target = stage
            .path()
            .join("source/raw")
            .join(format!("{}.json", path.replace('\\', "/")));
        fs::create_dir_all(target.parent().context("raw parent")?)?;
        fs::write(target, serde_json::to_vec_pretty(document)?)?;
    }
    let submission = release::submission(stage.path(), &package, &inventory)?;
    fs::write(
        stage.path().join("release-audit.json"),
        serde_json::to_vec_pretty(&inventory)?,
    )?;
    fs::write(
        stage.path().join("core-build.json"),
        serde_json::to_vec_pretty(&json!({"core":audit,"modio":receipt,"submission":submission}))?,
    )?;
    fs::rename(stage.path(), &args.out)?;
    println!(
        "{}",
        serde_json::to_string_pretty(
            &json!({"output":args.out,"registered_workspots":workspots,"archive_verified":receipt["archive_verified"],"zip_verified":receipt["zip_verified"],"runtime_verified":false})
        )?
    );
    Ok(())
}

fn main() -> Result<()> {
    // Native resource graphs contain deeply nested handles.
    std::thread::Builder::new()
        .name("quest-core-compiler".into())
        .stack_size(64 * 1024 * 1024)
        .spawn(|| build(Args::parse()))?
        .join()
        .map_err(|_| anyhow::anyhow!("quest core compiler panicked"))?
}
