use async_openai::types::{AssistantTools, AssistantToolsFunction, AssistantToolsRetrieval};
use ha_ai::openai::Environment;

#[tokio::main]
async fn main() {
  let mut tools = vec![AssistantTools::Retrieval(AssistantToolsRetrieval::default())];

  let glob_pattern = "resources/services/*.json";

  for entry in glob::glob(glob_pattern).expect("Failed to read glob pattern") {
    let Ok(path) = entry else {
      continue;
    };

    let service = path.file_stem().expect("Error getting file stem").to_str().expect("Error converting to str");
    let service_path = format!("resources/services/{}.json", service);
    let parameters = std::fs::read_to_string(service_path).expect("Error reading parameters");
    let function: AssistantToolsFunction = serde_json::from_str(&parameters).expect("Error parsing parameters");
    let tool = AssistantTools::Function(function);

    tools.push(tool);
  }

  let instructions = std::fs::read_to_string("resources/instructions.txt").expect("Error reading instructions");
  let env = Environment::new(tools, instructions);
  let runtime = env.runtime().await.expect("Error creating runtime");

  // runtime.ask("turn on all the lights").await.expect("Error calling runtime");
  runtime.ask("turn off all the lights").await.expect("Error calling runtime");
}
