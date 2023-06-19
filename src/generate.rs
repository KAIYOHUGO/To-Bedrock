use std::{collections::HashMap, path::Path};

use crate::{
    pack::{pack_addon, Header, LangInfo, Manifest, Module},
    parse::TranslateKV,
};
use anyhow::{anyhow, Result};
use tokio::task;

/// output is `Map<bedrock_id,java_id>`
fn gen_bedrock_java_id_map(bedrock: TranslateKV, java: TranslateKV) -> TranslateKV {
    let mut java_text_id = TranslateKV::new();
    java_text_id.reserve(java.capacity());
    for (k, v) in java {
        java_text_id.insert(v, k);
    }

    let mut ret = TranslateKV::new();
    for (bedrock_id, bedrock_text) in bedrock {
        if let Some(java_id) = java_text_id.get(&bedrock_text).cloned() {
            ret.insert(bedrock_id, java_id);
        }
    }
    ret
}

fn gen_translate(
    bedrock_java_id_map: &TranslateKV,
    mut java: TranslateKV,
    bedrock: TranslateKV,
) -> TranslateKV {
    let mut translate = TranslateKV::new();
    for (bedrock_id, java_id) in bedrock_java_id_map {
        let Some(java_text) = java.remove(java_id) else {
            continue;
        };
        if Some(&java_text) == bedrock.get(bedrock_id) {
            continue;
        }
        translate.insert(bedrock_id.clone(), java_text);
    }
    translate
}

pub async fn gen_output(
    mut java_texts: HashMap<String, TranslateKV>,
    mut bedrock_texts: HashMap<String, TranslateKV>,
    version: [u8; 3],
    output: &Path,
) -> Result<TranslateKV> {
    let (Some(java_en_us),Some(bedrock_en_us))=(java_texts.remove("en_us"),bedrock_texts.remove("en_us")) else {
        return Err(anyhow!("en_us.json and en_us.lang file are required"));
    };

    let bedrock_java_id_map = gen_bedrock_java_id_map(bedrock_en_us, java_en_us);
    let mut path = output.join(format!(
        "To_Bedrock_{}_{}_{}",
        &version[0], &version[1], &version[2]
    ));
    path.set_extension("mcpack");
    let manifest = Manifest {
        format_version: 2,
        header: Header {
            description: "Java Translation Pack Generate by kaiyo hugo".into(),
            name: "To Bedrock Translate Resource Pack".into(),
            uuid: "66c6e9a8-3093-462a-9c36-dbb052165623".into(),
            version,
            min_engine_version: [1, 16, 0],
        },
        modules: vec![Module {
            module_type: "resources".into(),
            uuid: "743f6949-53be-44b6-b326-398005028623".into(),
            version,
        }],
    };

    let mut lang_info_list = vec![];
    for (id, java_text) in java_texts {
        let id = id
            .split_once('_')
            .map(|(begin, end)| format!("{}_{}", begin, end.to_uppercase()))
            .unwrap_or(id);

        let name = {
            let name = java_text
                .get("language.name")
                .ok_or_else(|| anyhow!("cannot find name in java name file (malformed)"))?;
            let region = java_text
                .get("language.region")
                .ok_or_else(|| anyhow!("cannot find region in java name file (malformed)"))?;
            format!("{name} ({region})")
        };
        let bedrock_text = bedrock_texts.remove(&id).unwrap_or_default();
        let texts = gen_translate(&bedrock_java_id_map, java_text, bedrock_text);
        lang_info_list.push(LangInfo { id, name, texts })
    }
    task::spawn_blocking(|| pack_addon(path, lang_info_list, manifest)).await??;
    Ok(bedrock_java_id_map)
}
