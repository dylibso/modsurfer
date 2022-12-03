use anyhow::Result;
// use modserver_convert;
use modsurfer::Module;
use modsurfer_proto_v1::api;
use protobuf::Message;
use reqwest;

pub async fn get_module(host: &url::Url, id: i64) -> Result<Module> {
    let req = api::GetModuleRequest {
        module_id: id,
        ..Default::default()
    };

    let client = reqwest::Client::new();
    let res = client
        .post(host.join("api/v1/module")?)
        .body(req.write_to_bytes()?)
        .send()
        .await?;
    let data = res.bytes().await?;
    let resp: api::GetModuleResponse = protobuf::Message::parse_from_bytes(&data)?;
    if resp.error.is_some() {
        println!("error {:?}", resp.error);
    } else {
        let m = resp.module;
        println!(
            "{}, imports: {}, exports: {}, size: {}",
            m.location,
            m.imports.len(),
            m.exports.len(),
            m.size
        );
    }

    todo!()
}
