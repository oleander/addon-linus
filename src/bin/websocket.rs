use async_openai::types::{AssistantTools, AssistantToolsFunction};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use ha_ai::openai::Environment;
use tokio::net::TcpListener;
use ha_ai::shared;

#[tokio::main]
async fn main() {
  ctrlc::set_handler(move || {
    println!("Received Ctrl+C!");
    std::process::exit(0);
  }).expect("Error setting Ctrl+C handler");

  let mut tools = vec![];

  env_logger::init();

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

  println!("Loaded {} tools", tools.len());
  println!("Supervisor token: {}", *shared::SUPERVISOR_TOKEN);
  println!("OpenAi API key: {}", *shared::OPENAI_API_KEY);
  println!("Socket port: {}", *shared::SOCKET_PORT);

  let instructions = std::fs::read_to_string("resources/instructions.txt").expect("Error reading instructions");
  let env = Environment::new(tools, instructions);
  let runtime = env.runtime().await.expect("Error creating runtime");

  let listener = TcpListener::bind("0.0.0.0:10200").await.expect("Failed to bind TCP listener");
  println!("Listening on {}", listener.local_addr().unwrap());

  loop {
    let (mut socket, _) = listener.accept().await.expect("Failed to accept connection");
    let mut buffer = [0; 1024];
    match socket.read(&mut buffer).await {
      Ok(_) => {
        let request = String::from_utf8_lossy(&buffer);
        println!("Received request: {}", request);

        if request.trim().contains("ping") {
          println!("Received ping");
          if let Err(e) = socket.write_all("pong".as_bytes()).await {
            println!("Failed to send response: {}", e);
          } else {
            println!("Sent pong");
          }
        } else {
          match runtime.ask(request.trim()).await {
            Ok(response) => {
              if let Err(e) = socket.write_all(response.as_bytes()).await {
                println!("Failed to send response: {}", e);
              }
            }
            Err(e) => {
              println!("Failed to call runtime: {}", e);
              let msg = format!("Error calling runtime: {}", e);
              if let Err(e) = socket.write_all(msg.as_bytes()).await {
                println!("Failed to send response: {}", e);
              }
            }
          }
        }
      }
      Err(e) => println!("Failed to read from socket: {}", e)
    }
  }
}
