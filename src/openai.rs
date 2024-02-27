use async_openai::types::{
  AssistantObject, AssistantTools, CreateAssistantRequestArgs, CreateMessageRequestArgs, CreateRunRequestArgs, CreateThreadRequestArgs, MessageContent, RunObject, RunStatus, RunToolCallObject, SubmitToolOutputsRunRequest, ThreadObject, ToolsOutputs
};
use async_openai::config::OpenAIConfig;
use anyhow::{bail, Context, Result};
use futures::future::join_all;
use async_openai::Client;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::shared;

trait AsyncOperations {
// not sure this is a goos idea 
  async fn initiate_run(&self, assistant: &AssistantObject, thread: &ThreadObject, question: &str) -> Result<RunObject>;
  async fn create_assistant(&self, tools: &[AssistantTools], instructions: &str) -> Result<AssistantObject>;
  async fn handle_tool_call(&self, tool_call: &RunToolCallObject) -> Result<ToolsOutputs>;
  async fn event_loop(&self, thread: &ThreadObject, run: &RunObject) -> Result<String>;
  async fn clean_up(&self, xthread: &ThreadObject) -> Result<()>;
  async fn create_thread(&self) -> Result<ThreadObject>;
  async fn handle_generic_tool_call(&self, tool_call: &RunToolCallObject) -> Result<ToolsOutputs>;
  async fn handle_multi_tool_use_parallel(&self, tool_call: &RunToolCallObject) -> Result<ToolsOutputs>;
  async fn file_ids(&self) -> Result<Vec<String>>;
}

#[derive(Deserialize)]
struct MultiToolUseParallelArgs {
  tool_uses: Vec<ToolUse>
}

#[derive(Deserialize)]
struct ToolUse {
  parameters:     Value,
  recipient_name: String
}

impl AsyncOperations for Client<OpenAIConfig> {
  async fn create_thread(&self) -> Result<ThreadObject> {
    let thread_request = CreateThreadRequestArgs::default().build()?;
    self.threads().create(thread_request).await.context("Error creating thread")
  }

  async fn create_assistant(&self, tools: &[AssistantTools], instructions: &str) -> Result<AssistantObject> {
    let assistant_request = CreateAssistantRequestArgs::default()
      .name(shared::ASSISTANT_NAME)
      .instructions(instructions)
      .model(shared::MODEL)
      .tools(tools)
      .build()?;

    self.assistants().create(assistant_request).await.context("Error creating assistant")
  }

  async fn file_ids(&self) -> Result<Vec<String>> {
    Ok(crate::files::template_files().await?.into_iter().map(|file| file.id).collect::<Vec<String>>())
  }

  async fn initiate_run(&self, assistant: &AssistantObject, thread: &ThreadObject, question: &str) -> Result<RunObject> {
    let message = CreateMessageRequestArgs::default().role("user").content(question).build()?;
    let _message_obj = self.threads().messages(&thread.id).create(message).await?;
    let run_request = CreateRunRequestArgs::default().assistant_id(&assistant.id).build()?;
    self.threads().runs(&thread.id).create(run_request).await.context("Error creating run")
  }

  async fn event_loop(&self, thread: &ThreadObject, run: &RunObject) -> Result<String> {
    use RunStatus::*;

    loop {
      match self.threads().runs(&thread.id).retrieve(&run.id).await? {
        RunObject {
          status: Completed, ..
        } => {
          println!("--- Run Completed");

          let query = [("limit", shared::LIMIT.to_string())];
          let response = self.threads().messages(&thread.id).list(&query).await?;
          let message_id = response.data.first().context("No messages")?.id.clone();
          let message = self.threads().messages(&thread.id).retrieve(&message_id).await?;
          let content = message.content.first().context("No content")?;

          let MessageContent::Text(text) = content else {
            panic!("Only text is supported in the terminal");
          };

          println!("--- Response: {:#?}", text);
          return Ok(text.text.value.clone());
        }

        RunObject {
          status: Failed,
          last_error: Some(err),
          ..
        } => {
          bail!("--- Run Failed: {} ({:?})", err.message, err.code);
        }

        RunObject {
          status: RequiresAction,
          required_action: Some(action),
          ..
        } => {
          println!("--- Requires action");

          let tool_calls = action.submit_tool_outputs.tool_calls.clone();

          let mut futures = vec![];

          for tool_call in &tool_calls {
            futures.push(self.handle_tool_call(tool_call));
          }

          let submit_request = SubmitToolOutputsRunRequest {
            tool_outputs: join_all(futures).await.into_iter().collect::<Result<Vec<_>>>()?
          };

          println!("--- Submitting tool outputs: {:#?}", submit_request);
          self.threads().runs(&thread.id).submit_tool_outputs(&run.id, submit_request).await?;
        }

        RunObject {
          status: InProgress | Queued, ..
        } => {
          println!("--- Waiting for response...");
        }

        RunObject {
          status, ..
        } => {
          panic!("--- Unexpected run status: {:?}", status);
        }
      }
    }
  }

  async fn handle_tool_call(&self, tool_call: &RunToolCallObject) -> Result<ToolsOutputs> {
    match tool_call.function.name.as_str() {
      "multi_tool_use" => bail!("multi_tool_use is not supported"),
      "multi_tool_use.parallel" => self.handle_multi_tool_use_parallel(tool_call).await,
      _ => self.handle_generic_tool_call(tool_call).await
    }
  }

  // The modified handle_multi_tool_use_parallel function
  async fn handle_multi_tool_use_parallel(&self, tool_call: &RunToolCallObject) -> Result<ToolsOutputs> {
    let multi_tools = serde_json::from_str::<MultiToolUseParallelArgs>(&tool_call.function.arguments).context("Failed to parse arguments for multi_tool_use.parallel")?;
    let mut futures = vec![];

    for tool_use in multi_tools.tool_uses {
      let recipient_name = tool_use.recipient_name.clone();
      let parts: Vec<String> = recipient_name.split('.').map(|s| s.to_string()).collect();
      let domain = parts.get(1).cloned().context("Invalid service name")?.clone();
      let service = parts.get(2).cloned().context("Invalid service name")?.clone();
      let service_data = tool_use.parameters.clone();
      let lazy = crate::socket::call_service(domain, service, service_data);
      futures.push(lazy);
    }

    let output = join_all(futures).await.into_iter().collect::<Result<Vec<_>>>()?;
    let output_json = json!({
      "tool_outputs": output
    });

    Ok(ToolsOutputs {
      tool_call_id: tool_call.id.clone().into(),
      output:       output_json.to_string().into()
    })
  }

  async fn handle_generic_tool_call(&self, tool_call: &RunToolCallObject) -> Result<ToolsOutputs> {
    println!("--- handle_generic_tool_call: {:#?}", tool_call);

    let service_data = serde_json::from_str::<Value>(&tool_call.function.arguments).context("Invalid JSON")?;
    let (domain, service) = tool_call.function.name.split_once('.').context("Invalid service name")?;
    let response = crate::socket::call_service(domain.to_string(), service.to_string(), service_data).await?;

    Ok(ToolsOutputs {
      tool_call_id: tool_call.id.clone().into(),
      output:       response.into()
    })
  }

  async fn clean_up(&self, thread: &ThreadObject) -> Result<()> {
    self.threads().delete(&thread.id).await.context("Error deleting thread").map(|_| ())
  }
}

#[derive(Debug, Clone)]
pub struct Environment {
  client:       Client<OpenAIConfig>,
  tools:        Vec<AssistantTools>,
  instructions: String
}

impl Environment {
  pub fn new(tools: Vec<AssistantTools>, instructions: String) -> Self {
    Self {
      client: Client::new(),
      tools,
      instructions
    }
  }

  pub async fn runtime(&self) -> Result<Runtime> {
    let thread = self.client.create_thread().await.context("Error creating thread")?;
    let assistant = self.client.create_assistant(&self.tools, &self.instructions).await.context("Error creating assistant")?;

    Ok(Runtime {
      env: self.clone(),
      assistant,
      thread
    })
  }
}

#[derive(Debug, Clone)]
pub struct Runtime {
  env:       Environment,
  assistant: AssistantObject,
  thread:    ThreadObject
}

impl Runtime {
  pub async fn ask(&self, question: &str) -> Result<String> {
    println!("Asking: {}", question);

    let run = self.env.client.initiate_run(&self.assistant, &self.thread, question).await.context("Error initiating run")?;
    self.env.client.event_loop(&self.thread, &run).await
  }
}

// impl Drop for Runtime {
//   fn drop(&mut self) {
//     tokio_async_drop!({
//       self.env.client.clean_up(&self.thread).await.expect("Error cleaning up thread");
//     });
//   }
// }
