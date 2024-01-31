use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use anyhow::{Context, Result};
use handlebars::Handlebars;
use serde_json::json;
use ha_ai::shared;
use tokio::fs;

#[derive(Deserialize, Serialize, Debug)]
struct Attributes {
  pub friendly_name: Option<String>
}

#[derive(Deserialize, Serialize, Debug)]
struct Entity {
  pub attributes: Attributes,
  pub entity_id:  String,
  pub state:      String,
  pub domain:     Option<String>
}

use handlebars::{Context as HContext, Helper, HelperResult, Output, RenderContext};

fn domain(h: &Helper, _: &Handlebars, _: &HContext, _: &mut RenderContext, out: &mut dyn Output) -> HelperResult {
  let entity = h.param(0).unwrap().value().as_object().unwrap();
  let entity = serde_json::from_value::<Entity>(serde_json::Value::Object(entity.clone())).unwrap();
  let result = entity.entity_id.split('.').collect::<Vec<&str>>()[0].to_string();
  out.write(&result)?;
  Ok(())
}

lazy_static::lazy_static! {
  #[derive(Serialize, Debug, Clone)]
  static ref KEEP_ENTITIES: [String; 4] = [
    "light.bakrum_skarm_lampa_lampa".to_string(),
    "light.bakrum_bord_tak_lampa".to_string(),
    "light.bakrum_liten_lampa_lampa".to_string(),
    "light.bakrum_shelly_roof".to_string()
  ];
}

#[tokio::main]
async fn main() -> Result<()> {
  let url = "http://homeassistant.local:8123/api/states";
  let mut handlebars = Handlebars::new();
  let client = reqwest::Client::new();
  let mut data = BTreeMap::new();
  let response = client
    .get(url)
    .header("Authorization", format!("Bearer {}", *shared::SUPERVISOR_TOKEN))
    .header("Content-Type", "application/json")
    .send()
    .await?;
  let response = response.json::<Vec<Entity>>().await?;
  let exposed_entities: Vec<Entity> = response.into_iter().filter(|entity| KEEP_ENTITIES.contains(&entity.entity_id)).collect();
  let exposed_entity = exposed_entities.first().unwrap();
  println!("Exposed entities: {:#?}", exposed_entity);
  let template_string = fs::read_to_string("resources/templates/instructions.hbs").await?;
  handlebars.set_strict_mode(true);
  handlebars.register_helper("domain", Box::new(domain));
  handlebars.register_template_string("template", template_string).context("Error registering template")?;
  data.insert("exposed_entities", json!(exposed_entities));
  data.insert("now", json!(chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()));
  let rendered = handlebars.render("template", &data).context("Error rendering template")?;
  fs::write("resources/instructions.txt", rendered).await.context("Error writing instructions")?;
  Ok(())
}
