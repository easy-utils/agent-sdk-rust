pub mod agent {
    pub mod v1 {
        include!("agent/v1/agent.rs");
        include!("agent/v1/agent_easyrpc.rs");
    }
}

use easy_rpc::bridge_reqwest::NewClient;
use easy_rpc::protocol::{Headers, Request, Transport};
use prost::Message;

fn encode<M: Message>(m: &M) -> bytes::Bytes {
    m.encode_to_vec().into()
}

pub struct Client {
    base: String,
    token: String,
    transport: Option<std::sync::Arc<dyn Transport>>,
}

impl Client {
    pub fn new(base: &str, token: &str) -> Self {
        Self { base: base.to_string(), token: token.to_string(), transport: None }
    }

    fn transport(&self) -> std::sync::Arc<dyn Transport> {
        match &self.transport {
            Some(t) => t.clone(),
            None => std::sync::Arc::from(NewClient(self.base.clone())),
        }
    }

    fn headers(&self) -> Headers {
        let mut h = Headers::new();
        h.insert("Authorization".to_string(), vec![format!("Bearer {}", self.token)]);
        h
    }

    fn req(&self, url: &str, body: Vec<u8>) -> Request {
        Request {
            url: url.to_string(),
            method: "POST".to_string(),
            headers: self.headers(),
            body: Some(body.into()),
        }
    }


    pub async fn list_sessions(&self) -> Result<Vec<String>, String> {
        let t = self.transport();
        let req = self.req(
            "/agent.v1.AgentService/ListSessions",
            encode(&agent::v1::ListSessionsRequest {}).to_vec(),
        );
        let res = t.send(req).await.map_err(|e| e.to_string())?;
        if res.status >= 300 {
            return Err(res.error.map(|e| e.to_string()).unwrap_or_else(|| res.status.to_string()));
        }
        let out = agent::v1::ListSessionsResponse::decode(&res.body[..]).map_err(|e| e.to_string())?;
        Ok(out.sessions.iter().map(|s| s.name.clone()).collect())
    }


    pub async fn list_presets(&self, locale: &str) -> Result<Vec<String>, String> {
        let t = self.transport();
        let req = self.req(
            "/agent.v1.AgentService/ListPresets",
            encode(&agent::v1::ListPresetsRequest { locale: locale.to_string() }).to_vec(),
        );
        let res = t.send(req).await.map_err(|e| e.to_string())?;
        if res.status >= 300 {
            return Err(res.error.map(|e| e.to_string()).unwrap_or_else(|| res.status.to_string()));
        }
        let out = agent::v1::ListPresetsResponse::decode(&res.body[..]).map_err(|e| e.to_string())?;
        Ok(out.presets.iter().map(|p| p.id.clone()).collect())
    }

    pub async fn health(&self) -> Result<bool, String> {
        let t = self.transport();
        let req = self.req(
            "/agent.v1.AgentService/Health",
            encode(&agent::v1::HealthRequest {}).to_vec(),
        );
        let res = t.send(req).await.map_err(|e| e.to_string())?;
        if res.status >= 300 {
            return Err(res.error.map(|e| e.to_string()).unwrap_or_else(|| res.status.to_string()));
        }
        let out = agent::v1::HealthResponse::decode(&res.body[..]).map_err(|e| e.to_string())?;
        Ok(out.ok)
    }
}

impl Client {
    pub fn with_ca_pem(base: &str, token: &str, ca_pem: &[u8]) -> Result<Self, String> {
        let cert = reqwest::Certificate::from_pem(ca_pem).map_err(|e| e.to_string())?;
        let client = reqwest::Client::builder()
            .add_root_certificate(cert)
            .build()
            .map_err(|e| e.to_string())?;
        let t = easy_rpc::bridge_reqwest::ReqwestTransport::with_client(base.to_string(), client);
        Ok(Self { base: base.to_string(), token: token.to_string(), transport: Some(std::sync::Arc::new(t)) })
    }
}
