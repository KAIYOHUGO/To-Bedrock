//! Note
//! https://wiki.vg/Game_files
//! https://gist.github.com/skyrising/95a8e6a7287634e097ecafa2f21c240f

mod api;
mod fetch_bedrock;
mod fetch_java;
mod generate;
mod pack;
mod parse;

use anyhow::{anyhow, Result};

use clap::Parser;
use generate::gen_output;

use parse::{des_bedrock, des_java};
use std::{collections::HashMap, io::BufWriter, path::PathBuf};
use tokio::{
    fs, io,
    task::{self, spawn_blocking},
    try_join,
};

#[derive(Debug, Clone, Parser)]
struct Cli {
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Debug, Clone, clap::Subcommand)]
enum Cmd {
    Auto(AutoCmd),
    Raw(RawCmd),
}

#[derive(Debug, Clone, clap::Args)]
struct AutoCmd {
    /// java version
    #[arg(short, long)]
    java: String,

    /// bedrock version
    #[arg(short, long)]
    bedrock: Option<String>,

    /// output folder path
    #[arg(short, long)]
    output: PathBuf,

    /// emit bedrock java id map
    #[arg(long)]
    emit_map: bool,
}

#[derive(Debug, Clone, clap::Args)]
struct RawCmd {
    /// java texts folder path (require en_us.json)
    #[arg(short, long)]
    java: PathBuf,

    /// bedrock texts folder path (require en_us.lang)
    #[arg(short, long)]
    bedrock: PathBuf,

    /// output folder path
    #[arg(short, long)]
    output: PathBuf,

    /// pack (addon) version e.g. `1.19.0`
    #[arg(short, long)]
    pack_version: String,

    /// emit bedrock java id map
    #[arg(long)]
    emit_map: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let Cli { command } = Cli::parse();

    match command {
        Cmd::Auto(cmd) => auto_cmd(cmd).await?,
        Cmd::Raw(cmd) => raw_cmd(cmd).await?,
    };
    Ok(())
}

async fn auto_cmd(cmd: AutoCmd) -> Result<()> {
    let bedrock_version = cmd.bedrock.unwrap_or_else(|| cmd.java.clone());

    let java_package = {
        let java_version = api::Id(cmd.java.clone());
        let manifest = api::get_version_manifest().await?;
        let version = manifest
            .versions
            .into_iter()
            .find(|version| version.id == java_version)
            .ok_or_else(|| anyhow!("Cannot find version `{}`", &java_version.0))?;
        version.url.get().await?
    };

    let java_texts = fetch_java::fetch(java_package, 10.try_into()?).await?;
    let bedrock_texts = fetch_bedrock::fetch(bedrock_version, 10.try_into()?).await?;

    let bedrock_java_id_map = gen_output(
        java_texts,
        bedrock_texts,
        parse_version(cmd.java)?,
        &cmd.output,
    )
    .await?;
    if cmd.emit_map {
        spawn_blocking(move || {
            let file = BufWriter::new(std::fs::File::create(cmd.output.join("map.json"))?);
            serde_json::to_writer(file, &bedrock_java_id_map)?;
            anyhow::Ok(())
        })
        .await??;
    }
    Ok(())
}

async fn raw_cmd(cmd: RawCmd) -> Result<()> {
    let java = async {
        let mut ret = HashMap::new();
        let mut dir = fs::read_dir(cmd.java).await?;
        while let Some(file_meta) = dir.next_entry().await? {
            let name: PathBuf = file_meta.file_name().into();

            let Some(ext) = name.extension() else {
                continue;
            };
            if ext.to_str() != Some("json") {
                continue;
            }
            if !file_meta.file_type().await?.is_file() {
                continue;
            }
            let lang_id = name
                .with_extension("")
                .to_str()
                .ok_or_else(|| anyhow!("non UTF8 char in the file name"))?
                .to_owned();
            let file = fs::read(file_meta.path()).await?;
            let reader: &[u8] = file.as_ref();
            let kv = des_java(reader)?;
            ret.insert(lang_id, kv);
        }
        anyhow::Ok(ret)
    };
    let bedrock = async {
        let mut ret = HashMap::new();
        let mut dir = fs::read_dir(cmd.bedrock).await?;
        while let Some(file_meta) = dir.next_entry().await? {
            let name: PathBuf = file_meta.file_name().into();

            let Some(ext) = name.extension() else {
                continue;
            };
            if ext.to_str() != Some("lang") {
                continue;
            }
            if !file_meta.file_type().await?.is_file() {
                continue;
            }
            let lang_id = name
                .with_extension("")
                .to_str()
                .ok_or_else(|| anyhow!("non UTF8 char in the file name"))?
                .to_owned();
            let file = fs::File::open(file_meta.path()).await?;
            let kv = des_bedrock(io::BufReader::new(file)).await?;
            ret.insert(lang_id, kv);
        }
        anyhow::Ok(ret)
    };
    let (java_texts, bedrock_texts) = try_join!(task::spawn(java), task::spawn(bedrock))?;
    let java_texts = java_texts?;
    let bedrock_texts = bedrock_texts?;

    let bedrock_java_id_map = gen_output(
        java_texts,
        bedrock_texts,
        parse_version(cmd.pack_version)?,
        &cmd.output,
    )
    .await?;
    if cmd.emit_map {
        spawn_blocking(move || {
            let file = BufWriter::new(std::fs::File::create(cmd.output.join("map.json"))?);
            serde_json::to_writer(file, &bedrock_java_id_map)?;
            anyhow::Ok(())
        })
        .await??;
    }
    Ok(())
}

fn parse_version(version: String) -> Result<[u8; 3]> {
    let mut ret = version
        .split('.')
        .map(|x| x.parse())
        .collect::<Result<Vec<u8>, _>>()?;
    while ret.len() < 3 {
        ret.push(0);
    }
    ret.try_into().map_err(|_| anyhow!("Version Format Error"))
}

#[cfg(test)]
mod tests {

    #[tokio::test]
    async fn test_name() {
        // let mut ret = HashMap::new();
        let octocrab = octocrab::instance();
        let page = octocrab
            .repos("Mojang", "bedrock-samples")
            .get_content()
            .path("resource_pack/texts")
            .r#ref(format!("v{}", "1.19.70.2"))
            .send()
            .await
            .unwrap();
        dbg!(page);
    }
}
