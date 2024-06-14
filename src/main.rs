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

use clap::{
    builder::{styling::AnsiColor, Styles},
    Parser,
};
use generate::gen_output;

use parse::{des_bedrock, des_java, TranslateKV};
use std::{collections::HashMap, io::BufWriter, path::PathBuf};
use tokio::{
    fs, io,
    task::{self, spawn_blocking},
    try_join,
};

#[derive(Debug, Clone, Parser)]
#[command(styles = clap_v3_styles())]
struct Cli {
    #[command(subcommand)]
    command: Cmd,

    /// output folder path
    #[arg(short, long)]
    output: PathBuf,

    /// Override bedrock java id map
    /// It is useful when some mapping is wrong or missing
    #[arg(long)]
    override_map: Option<PathBuf>,

    /// Emit bedrock java id map
    /// It is a key (bedrock text id) value (java text id) map
    #[arg(long)]
    emit_map: bool,
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
}

#[derive(Debug, Clone, clap::Args)]
struct RawCmd {
    /// java texts folder path (require en_us.json)
    #[arg(short, long)]
    java: PathBuf,

    /// bedrock texts folder path (require en_us.lang)
    #[arg(short, long)]
    bedrock: PathBuf,

    /// pack (addon) version e.g. `1.19.0`
    #[arg(short, long)]
    pack_version: String,
}

fn clap_v3_styles() -> Styles {
    Styles::styled()
        .header(AnsiColor::Yellow.on_default())
        .usage(AnsiColor::Green.on_default())
        .literal(AnsiColor::Green.on_default())
        .placeholder(AnsiColor::Green.on_default())
}

#[tokio::main]
async fn main() -> Result<()> {
    let Cli {
        command,
        output,
        override_map,
        emit_map,
    } = Cli::parse();

    let cmd_output = match command {
        Cmd::Auto(cmd) => auto_cmd(cmd).await?,
        Cmd::Raw(cmd) => raw_cmd(cmd).await?,
    };

    let override_map = if let Some(path) = override_map {
        let file = fs::read(path).await?;
        serde_json::from_reader(file.as_slice())?
    } else {
        TranslateKV::new()
    };

    let bedrock_java_id_map = gen_output(
        cmd_output.java_texts,
        cmd_output.bedrock_texts,
        override_map,
        cmd_output.version,
        &output,
    )
    .await?;

    if emit_map {
        spawn_blocking(move || {
            let file = BufWriter::new(std::fs::File::create(output.join("map.json"))?);
            serde_json::to_writer(file, &bedrock_java_id_map)?;
            anyhow::Ok(())
        })
        .await??;
    }
    Ok(())
}

struct CmdOutput {
    java_texts: HashMap<String, TranslateKV>,
    bedrock_texts: HashMap<String, TranslateKV>,
    version: [u8; 3],
}

async fn auto_cmd(cmd: AutoCmd) -> Result<CmdOutput> {
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
    let version = parse_version(cmd.java)?;

    Ok(CmdOutput {
        java_texts,
        bedrock_texts,
        version,
    })
}

async fn raw_cmd(cmd: RawCmd) -> Result<CmdOutput> {
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
    let version = parse_version(cmd.pack_version)?;

    Ok(CmdOutput {
        java_texts,
        bedrock_texts,
        version,
    })
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
