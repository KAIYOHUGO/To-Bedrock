use serde::Serialize;

#[derive(Serialize)]
pub struct Welcome {
    format_version: [u8; 3],
    header: Header,
    modules: Vec<Module>,
}

#[derive(Serialize)]
pub struct Header {
    description: String,
    name: String,
    uuid: String,
    version: [u8; 3],
    min_engine_version: [u8; 3],
}

#[derive(Serialize)]
pub struct Module {
    description: String,
    #[serde(rename = "type")]
    module_type: String,
    uuid: String,
    version: [u8; 3],
}
