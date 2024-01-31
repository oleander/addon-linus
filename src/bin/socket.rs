use serde_json::json;
use ha_ai::socket;

#[tokio::main]
async fn main() {
  let domain = "light";
  let service = "toggle";
  let entity_id = "light.bakrum_skarm_lampa_lampa";

  for n in 0..10 {
    let data = json!({
      "entity_id": entity_id,
      "brightness": n * 25
    });

    println!("[{}] Calling service: {} {}", n, domain, service);

    let res = socket::call_service(domain.to_string(), service.to_string(), data).await;

    println!("Response: {:?}", res);

    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
  }
}
