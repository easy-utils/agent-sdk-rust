// Live check against the local agent (supervisor agent-server :18080).
use agent_sdk_rust::Client;
use agent_sdk_rust::agent::v1::{WatchSessionsRequest, method_specs};
use easy_rpc::protocol::Request;
use prost::Message;

#[tokio::main]
async fn main() {
    let c = Client::new("http://127.0.0.1:18080", "devtoken");
    println!("health = {:?}", c.health().await);
    println!("presets(zh) = {:?}", c.list_presets("zh").await);

    // New stream RPC via the generated method specs (WatchSessions).
    let t = easy_rpc::bridge_reqwest::NewClient("http://127.0.0.1:18080".to_string());
    let specs = method_specs();
    let spec = specs.iter().find(|m| m.name == "WatchSessions").expect("spec");
    println!("WatchSessions path = {} server_stream={}", spec.path, spec.server_stream);
    let mut h = easy_rpc::protocol::Headers::new();
    h.insert("authorization".to_string(), vec!["Bearer devtoken".to_string()]);
    let req = Request { url: spec.path.clone(), headers: h, body: Some(WatchSessionsRequest{}.encode_to_vec().into()) };
    let start = std::time::Instant::now();
    let mut st = t.open_stream(req).await.expect("open_stream");
    let first = st.recv().await;
    println!("WatchSessions first frame after {:?} (ok={})", start.elapsed(), first.is_some());
}
