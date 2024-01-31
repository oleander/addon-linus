use async_openai::types::{AssistantTools, AssistantToolsFunction};
use ha_ai::openai::Environment;

#[tokio::main]
async fn main() {
  let question = "if a light is on, turn it off, otherwise turn it on";
  let parameters = std::fs::read_to_string("resources/services/light.toggle.json").expect("Error reading parameters");
  let function: AssistantToolsFunction = serde_json::from_str(&parameters).expect("Error parsing parameters");
  let instructions = std::fs::read_to_string("resources/instructions.txt").expect("Error reading instructions");
  let tool = AssistantTools::Function(function);
  let tools = vec![tool];

  let env = Environment::new(tools, instructions);
  let runtime = env.runtime().await.expect("Error creating runtime");

  runtime.ask(question).await.expect("Error calling runtime");
}
