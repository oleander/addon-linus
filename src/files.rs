use std::collections::HashMap;

use async_openai::types::OpenAIFile;
use async_openai::Client;
use anyhow::{Context, Result};

pub async fn template_files() -> Result<Vec<OpenAIFile>> {
  let glob_pattern = "resources/services/*.json";
  let all_files = all_files().await?;
  let mut found_files = vec![];

  println!("All files: {:#?}", all_files);

  for entry in glob::glob(glob_pattern).context("Failed to read glob pattern")? {
    let Ok(path) = entry else {
      continue;
    };

    let relative_path = path.file_name().context("Error getting file name")?.to_str().context("Error converting to str")?.to_string();

    let Some(found_file) = all_files.get(&relative_path) else {
      continue;
    };

    found_files.push(found_file.clone());
  }

  Ok(found_files)
}

pub async fn all_files() -> Result<HashMap<String, OpenAIFile>> {
  let client = Client::new();
  let query = [("purpose", "assistants")];
  let files = client.files().list(&query).await?;
  let mut table = HashMap::new();

  for file in files.data {
    table.insert(file.filename.clone(), file);
  }

  Ok(table)
}

// async fn upload_file(&self, path: &str) -> Result<String> {
//   let request = CreateFileRequestArgs::default().purpose("assistants").file(path).build().unwrap();
//   self.files().create(request).await.context("Error creating file").map(|file| file.id)
// }
