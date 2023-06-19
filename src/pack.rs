use anyhow::Result;
use serde::Serialize;
use std::{io::Write, path::Path};
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

pub struct LangInfo {
    pub id: String,
    pub name: String,
    pub texts: TranslateKV,
}

pub fn pack_addon(
    path: impl AsRef<Path>,
    lang_info_list: Vec<LangInfo>,
    manifest: Manifest,
) -> Result<()> {
    let mut file = ZipWriter::new(std::fs::File::create(path)?);
    let option = FileOptions::default().compression_level(Some(9));

    file.start_file("pack_icon.png", option)?;
    file.write_all(include_bytes!("../assets/pack_icon.png"))?;

    file.start_file("manifest.json", option)?;
    serde_json::to_writer(&mut file, &manifest)?;

    let mut lang_id_list = vec![];
    let mut lang_name_list = vec![];
    for LangInfo { id, name, texts } in &lang_info_list {
        lang_id_list.push(id);
        lang_name_list.push([id, name]);

        file.start_file(format!("texts/{}.lang", &id), option)?;
        sync_ser_bedrock(&mut file, texts)?;
    }

    file.start_file("texts/languages.json", option)?;
    serde_json::to_writer(&mut file, &lang_id_list)?;

    file.start_file("texts/language_names.json", option)?;
    serde_json::to_writer(&mut file, &lang_name_list)?;

    file.finish()?;
    Ok(())
}
