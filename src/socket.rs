use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use anyhow::{Context, Result};
use lazy_static::lazy_static;
use serde_json::Value;

use crate::shared;

lazy_static! {
  static ref HEADERS: HeaderMap<HeaderValue> = {
    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, format!("Bearer {}", *shared::SUPERVISOR_TOKEN).parse().expect("Error parsing authorization"));
    headers.insert(CONTENT_TYPE, "application/json".parse().expect("Error parsing content type"));
    headers
  };
}
mod response {
  use serde::{Deserialize, Serialize};
  use serde_json::Value;

  #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
  pub struct Response {
    response: Vec<Element>
  }

  #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
  pub struct Element {
    attributes: Value,
    context:    Context,
    entity_id:  String
  }

  #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
  pub struct Context {
    id:        String,
    parent_id: Option<Value>,
    user_id:   String
  }
}

pub async fn call_service(domain: String, service: String, service_data: Value) -> Result<String> {
  // let url = format!("http://homeassistant.local:8123/api/services/{domain}/{service}");
  let url = format!("http://supervisor/core/api/services/{domain}/{service}");
  let response = reqwest::Client::new().post(url).json(&service_data).headers(HEADERS.clone()).send().await?;

  println!("response (1): {:#?}", response);
  println!("\t--- Service called: {} {}", domain, service);
  println!("\t--- Service data: {}", service_data);

  let response = response.text().await;

  println!("response (2): {:#?}", response);

  response.context("Error calling service")
}
