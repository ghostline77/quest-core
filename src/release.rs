//! Explicit core payload inventory and separate cloud-cooking submission layout.
use anyhow::{Context, Result, ensure};
use ghostline_quest_builder::{evidence::digest, native::Binaries, package::Package, quest_core};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::{Read, Write},
    path::Path,
};
use zip::{ZipArchive, ZipWriter, write::SimpleFileOptions};

const OVERRIDES: [&str; 4] = [
    r"base\open_world\scenes\generic_vendors\versions\gold\chinese_food_store.scene",
    r"base\quest\persistent_content.questphase",
    r"base\worlds\03_night_city\_compiled\default\always_loaded_1.streamingsector",
    r"base\worlds\03_night_city\_compiled\default\exterior_-10_16_0_1.streamingsector",
];

pub(super) fn audit(binaries: &Binaries) -> Result<Value> {
    let mut expected = OVERRIDES
        .iter()
        .map(|p| (*p).to_owned())
        .collect::<BTreeSet<_>>();
    for name in [
        "controller.questphase",
        "health.questphase",
        "core.journal",
        r"localization\en-us\core.json",
    ] {
        expected.insert(format!(r"mod\ghostline_quest_core\{name}"));
    }
    for slot in 1..=quest_core::SLOT_COUNT {
        expected.insert(quest_core::slot_resource(slot)?);
    }
    for name in ["all.streamingblock", "start.scene", "world.streamingsector"] {
        expected.insert(format!(r"mod\quest_bootstrap\ghostline_quest_core\{name}"));
    }
    let actual = binaries.keys().cloned().collect::<BTreeSet<_>>();
    ensure!(
        actual == expected,
        "Core release inventory mismatch: unexpected={:?}; missing={:?}",
        actual.difference(&expected).collect::<Vec<_>>(),
        expected.difference(&actual).collect::<Vec<_>>()
    );
    Ok(
        json!({"format":"ghostline-quest-core-release-audit","resources":binaries.iter().map(|(p,b)|(p.clone(),digest(b))).collect::<BTreeMap<_,_>>(),"resource_count":actual.len(),"vanilla_overrides":OVERRIDES,"development_payloads":[],"runtime_dependencies":[],"allowlist_verified":true}),
    )
}

pub(super) fn submission(stage: &Path, package: &Package, inventory: &Value) -> Result<Value> {
    let local = stage.join("modio").join(&package.name);
    let root = stage.join("submission");
    let prefix = "archive/pc/mod";
    let files = BTreeMap::from([
        (
            format!("{prefix}/{}.archive", package.name),
            fs::read(local.join(format!("{}.archive", package.name)))?,
        ),
        (
            format!("{prefix}/{}.archive.xl", package.name),
            serde_json::to_vec_pretty(&package.registration)?,
        ),
    ]);
    fs::create_dir_all(&root)?;
    let zip_path = root.join(format!("{}-source.zip", package.name));
    let mut zip = ZipWriter::new(fs::File::create(&zip_path)?);
    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .last_modified_time(zip::DateTime::default());
    for (name, bytes) in &files {
        let target = root.join("project").join(name);
        fs::create_dir_all(target.parent().context("submission file parent")?)?;
        fs::write(target, bytes)?;
        zip.start_file(name, options)?;
        zip.write_all(bytes)?;
    }
    zip.finish()?;
    let mut zip = ZipArchive::new(fs::File::open(&zip_path)?)?;
    ensure!(
        zip.len() == files.len(),
        "Submission ZIP contains extra files"
    );
    let mut seen = BTreeSet::new();
    for i in 0..zip.len() {
        let mut entry = zip.by_index(i)?;
        let name = entry.name().to_owned();
        ensure!(seen.insert(name.clone()), "Duplicate submission path");
        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes)?;
        ensure!(
            files.get(&name) == Some(&bytes),
            "Submission ZIP byte mismatch: {name}"
        );
    }
    let receipt = json!({"layout":"redmod-source-project","zip":"submission/zz_ghostline_quest_core-source.zip","files":files.iter().map(|(p,b)|(p.clone(),digest(b))).collect::<BTreeMap<_,_>>(),"zip_sha256":digest(&fs::read(&zip_path)?),"zip_verified":true,"resource_count":inventory["resource_count"],"contains_scripts":false,"cloud_cooking_required":true,"modio_profile_metadata_is_separate":true});
    fs::write(
        root.join("verification.json"),
        serde_json::to_vec_pretty(&receipt)?,
    )?;
    Ok(receipt)
}
