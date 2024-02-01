use std::env::var;

#[derive(Debug, serde::Deserialize)]
struct Options {
  pub openai_api_key: String
}

lazy_static::lazy_static! {
  pub static ref SUPERVISOR_TOKEN: String = var("SUPERVISOR_TOKEN").expect("please set up the SUPERVISOR_TOKEN env variable before running this");
  // pub static ref SOCKET_PORT: String = var("SOCKET_PORT").expect("please set up the SOCKET_PORT env variable before running this");

  pub static ref OPENAI_API_KEY: String = {
    if let Ok(key) = var("OPENAI_API_KEY") {
      return key
    }

    match std::fs::read_to_string("/data/options.json") {
      Ok(content) => {
        match serde_json::from_str::<Options>(&content) {
          Ok(options) => {
            std::env::set_var("OPENAI_API_KEY", &options.openai_api_key);
            return options.openai_api_key;
          },
          Err(err) => {
            panic!("Error parsing /data/options.json file: {}", err);
          }
        }
      },
      Err(err) => {
        panic!("Error reading /data/options.json file: {}", err);
      }
    }
  };
}

pub static MODEL: &str = "gpt-3.5-turbo-1106";
pub static ASSISTANT_NAME: &str = "Linus HA";
pub static LIMIT: usize = 10;
