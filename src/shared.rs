use std::env::var;

lazy_static::lazy_static! {
  pub static ref SUPERVISOR_TOKEN: String = var("SUPERVISOR_TOKEN").expect("please set up the SUPERVISOR_TOKEN env variable before running this");
  pub static ref SOCKET_PORT: String = var("SOCKET_PORT").expect("please set up the SOCKET_PORT env variable before running this");
}

pub static MODEL: &str = "gpt-3.5-turbo-1106";
pub static ASSISTANT_NAME: &str = "Linus HA";
pub static LIMIT: usize = 10;
