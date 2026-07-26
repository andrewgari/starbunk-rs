use schemars::{schema_for, JsonSchema};

#[derive(JsonSchema)]
pub enum Addressee {
    Room,
    OtherUser,
    Self_,
}

#[derive(JsonSchema)]
pub struct TagResult {
    pub addressee: Option<Addressee>,
}

fn main() {
    let mut settings = schemars::gen::SchemaSettings::openapi3();
    settings.inline_subschemas = true;
    let gen = settings.into_generator();
    let schema = gen.into_root_schema_for::<TagResult>();
    println!("{}", serde_json::to_string_pretty(&schema).unwrap());
}
