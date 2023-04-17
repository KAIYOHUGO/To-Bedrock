use std::{collections::HashMap, path::PathBuf, sync::Arc};

use crate::parse::{ser_bedrock, TranslateKV};
use anyhow::{anyhow, Result};
use tokio::{fs, task};

/// output is `Map<bedrock_id,java_id>`
pub fn gen_bedrock_java_id_map(mut bedrock: TranslateKV, java: TranslateKV) -> TranslateKV {
    let mut java_value_key = TranslateKV::new();
    java_value_key.reserve(java.capacity());
    for (k, v) in java {
        java_value_key.insert(v, k);
    }

    for (_, v) in bedrock.iter_mut() {
        if let Some(java_key) = java_value_key.remove(v) {
            *v = java_key;
        }
    }
    bedrock
}

pub fn gen_translate(
    bedrock_java_id_map: &TranslateKV,
    mut java: TranslateKV,
    bedrock: TranslateKV,
) -> TranslateKV {
    let mut translate = TranslateKV::new();
    for (bedrock_id, bedrock_text) in bedrock {
        let Some(java_id) = bedrock_java_id_map.get(&bedrock_id) else {
            continue;
        };
        let Some(java_text) = java.remove(java_id) else {
            continue;
        };
        if bedrock_text == java_text {
            continue;
        }
        translate.insert(bedrock_id, java_text);
    }
    translate
}

pub async fn gen_output(
    mut java_texts: HashMap<String, TranslateKV>,
    mut bedrock_texts: HashMap<String, TranslateKV>,
    output: PathBuf,
) -> Result<()> {
    let (Some(java_en_us),Some(bedrock_en_us))=(java_texts.remove("en_us"),bedrock_texts.remove("en_us")) else {
        for k in java_texts.keys() {
            dbg!(k);
        }
        for k in bedrock_texts.keys() {
            dbg!(k);
        }
        return Err(anyhow!("en_us.json and en_us.lang file are required"));
    };

    let mut tasks = task::JoinSet::new();
    let id_map = Arc::new(gen_bedrock_java_id_map(bedrock_en_us, java_en_us));
    for (id, java_text) in java_texts {
        let Some(bedrock_text) = bedrock_texts.remove(&id) else {
            continue;
        };
        let id = id
            .split_once('_')
            .map(|(begin, end)| format!("{}_{}", begin, end.to_uppercase()))
            .unwrap_or(id);
        let id_map = id_map.clone();
        let mut path = output.join(id);
        path.set_extension("lang");
        tasks.spawn(async move {
            let text = gen_translate(&id_map, java_text, bedrock_text);
            let mut file = fs::File::create(path).await?;
            ser_bedrock(&mut file, text).await?;
            anyhow::Ok(())
        });
    }
    while let Some(next) = tasks.join_next().await {
        next??;
    }
    Ok(())
}
