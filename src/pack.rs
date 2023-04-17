use anyhow::Result;
use serde::Serialize;
use std::{io::Write, path::Path, sync::Arc};
use zip::{write::FileOptions, ZipWriter};

use crate::parse::{sync_ser_bedrock, TranslateKV};

#[derive(Serialize)]
pub struct Manifest {
    pub format_version: u8,
    pub header: Header,
    pub modules: Vec<Module>,
}

#[derive(Serialize)]
pub struct Header {
    pub description: String,
    pub name: String,
    pub uuid: String,
    pub version: [u8; 3],
    pub min_engine_version: [u8; 3],
}

#[derive(Serialize)]
pub struct Module {
    #[serde(rename = "type")]
    pub module_type: String,
    pub uuid: String,
    pub version: [u8; 3],
}

pub fn pack_addon(
    path: impl AsRef<Path>,
    lang_id: String,
    text: TranslateKV,
    manifest: Arc<Manifest>,
) -> Result<()> {
    let mut file = ZipWriter::new(std::fs::File::create(path)?);
    let option = FileOptions::default().compression_level(Some(9));

    file.start_file("pack_icon.png", option)?;
    file.write_all(include_bytes!("../assets/pack_icon.png"))?;

    file.start_file("manifest.json", option)?;
    serde_json::to_writer(&mut file, &*manifest)?;

    file.start_file(format!("texts/{}.lang", lang_id), option)?;
    sync_ser_bedrock(&mut file, text)?;

    file.finish()?;
    Ok(())
}
