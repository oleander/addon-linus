use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
  let files = ha_ai::files::template_files().await?;
  println!("Found {} template files", files.len());

  for file in files {
    println!("Found template file: {:#?}", file);
  }

  Ok(())
}
