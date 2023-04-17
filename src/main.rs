//! Note
//! https://wiki.vg/Game_files
//! https://gist.github.com/skyrising/95a8e6a7287634e097ecafa2f21c240f

mod api;
mod generate;
mod pack;
mod parse;

use anyhow::{anyhow, Result};
use api::get;
use clap::Parser;
use generate::gen_output;
use octocrab::params::repos::Reference;
use parse::{des_bedrock, des_en_us_from_java, des_java};
use std::{collections::HashMap, path::PathBuf};
use tokio::{fs, io, select, task, try_join};
use tokio_util::io::SyncIoBridge;

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
    let java_version = api::Id(cmd.java);

    let java = async move {
        let manifest = api::get_version_manifest().await?;
        let version = manifest
            .versions
            .into_iter()
            .find(|version| version.id == java_version)
            .ok_or_else(|| anyhow!("Cannot find version `{}`", &java_version.0))?;
        let package = version.url.get().await?;
        let mut set = task::JoinSet::new();

        set.spawn(async move {
            let java = package.downloads.client.url.get().await?;
            let kv = des_en_us_from_java(java, Some(package.downloads.client.size)).await?;
            dbg!("en_us");
            anyhow::Ok(("en_us".into(), kv))
        });

        let assets = package.asset_index.url.get().await?;
        let iter = assets
            .objects
            .into_iter()
            .filter(|(path, _)| path.starts_with("minecraft/lang/"))
            .map(|(path, obj)| {
                (
                    path.trim_start_matches("minecraft/lang/")
                        .trim_end_matches(".json")
                        .into(),
                    obj,
                )
            });

        for (lang_id, obj) in iter {
            set.spawn(async move {
                let reader = obj.url().get().await?;
                let kv =
                    task::spawn_blocking(move || des_java(SyncIoBridge::new(reader))).await??;
                dbg!(&lang_id);
                anyhow::Ok((lang_id, kv))
            });
        }

        anyhow::Ok(set)
    };
    let bedrock = async move {
        let mut set = task::JoinSet::new();
        let octocrab = octocrab::instance();
        let page = octocrab
            .repos("Mojang", "bedrock-samples")
            .get_content()
            .path("resource_pack/texts")
            .r#ref(format!("v{}", bedrock_version))
            .send()
            .await?;
        for i in page.items {
            let Some((lang_id,ext))=i.name.split_once(".") else {
                continue;
            };
            if ext != "lang" {
                continue;
            }
            let Some(url) = i.download_url else {
                continue;
            };
            let lang_id = lang_id.to_lowercase();
            set.spawn(async move {
                dbg!(&lang_id);
                let content = get(url).await?;
                let kv = des_bedrock(content).await?;
                anyhow::Ok((lang_id, kv))
            });
        }

        anyhow::Ok(set)
    };

    let (java_set, bedrock_set) = try_join!(task::spawn(java), task::spawn(bedrock))?;
    let (mut java_set, mut bedrock_set) = (java_set?, bedrock_set?);

    let (mut java_texts, mut bedrock_texts) = (HashMap::new(), HashMap::new());
    loop {
        select! {
            Some(r) = java_set.join_next() => {
                let (lang_id, kv) = r??;
                java_texts.insert(lang_id, kv);
            }
            Some(r) = bedrock_set.join_next() => {
                let (lang_id, kv) = r??;
                bedrock_texts.insert(lang_id, kv);
            }
            else => break,
        };
    }

    gen_output(java_texts, bedrock_texts, cmd.output).await
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

    gen_output(java_texts, bedrock_texts, cmd.output).await
}

#[cfg(test)]
mod tests {
    use super::*;

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
