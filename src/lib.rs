use std::{collections::HashMap, default};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use anyhow::{anyhow, Result};
use reqwest::{Client, ClientBuilder, Method};

pub mod structs;
pub use reqwest::Proxy;

pub struct Config {
    token: String,
    address: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            token: read_env("CONSUL_TOKEN", ""),
            address: read_env("CONSUL_ADDRESS", "http://127.0.0.1:8500"),
        }
    }
}

pub struct AgentBuilder {
    cfg: Config,
    proxies: Vec<Proxy>,
}

impl AgentBuilder {
    pub fn new(cfg: Config) -> AgentBuilder {
        AgentBuilder {
            cfg,
            proxies: vec![],
        }
    }

    pub fn with_proxy(mut self, proxy: Proxy) -> Self {
        self.proxies.push(proxy);
        self
    }

    pub fn build(self) -> Result<AgentClient> {
        let mut builder = ClientBuilder::new();

        for proxy in self.proxies {
            // add proxy
            builder = builder.proxy(proxy)
        }

        Ok(AgentClient {
            cfg: self.cfg,
            requester: builder.build()?,
            #[cfg(feature = "v1")]
            prefix: "/v1".to_string(),
        })
    }
}

#[derive(Debug,Default, Serialize, Deserialize)]
pub struct FilterRequest {
    pub filter: Option<String>,

    #[cfg(feature = "enterprise")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ns: Option<String>,
}

#[derive(Debug, Default,Serialize, Deserialize)]
pub struct DeregisterCheckRequest {
	#[serde(skip_serializing)]
    pub check_id: String,

    #[cfg(feature = "enterprise")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ns: Option<String>,
}

#[derive(Debug, Default,Serialize, Deserialize)]
pub struct AgentTTLCheckRequest {
	#[serde(skip_serializing)]
    pub check_id: String,

    pub note: Option<String>,

    #[cfg(feature = "enterprise")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ns: Option<String>,
}

#[derive(Debug, Default,Serialize, Deserialize)]
pub struct AgentTTLCheckUpdateRequest {
	#[serde(skip_serializing)]
    pub check_id: String,

    #[cfg(feature = "enterprise")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ns: Option<String>,

	#[serde(skip_serializing)]
    pub status: Option<String>,

	#[serde(skip_serializing)]
    pub output: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct AgentTTLCheckUpdateRequestBody {
	#[serde(rename = "Status")]
    status: Option<String>,

	#[serde(rename = "Output")]
    output: Option<String>,
}

#[derive(Debug, Default,Serialize, Deserialize)]
pub struct ServiceRequest {
	#[serde(skip_serializing)]
    pub service_id: String,

    #[cfg(feature = "enterprise")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ns: Option<String>,
}

pub struct AgentClient {
    cfg: Config,
    requester: Client,
    prefix: String,
}

impl AgentClient {
    pub fn new() -> Self {
        AgentBuilder::new(Config::default()).build().unwrap()
    }

    /// List Checks
    /// This endpoint returns all checks that are registered with the local agent.
    /// These checks were either provided through configuration files or added
    /// dynamically using the HTTP API.
    pub async fn agent_checks(&self, req: FilterRequest) -> Result<HashMap<String, structs::HealthCheck>> {
        self.execute_request(Method::GET, "/agent/checks",  &req, &()).await
    }

    /// Register Check
    /// This endpoint adds a new check to the local agent. Checks may be of script,
    /// HTTP, TCP, UDP, or TTL type. The agent is responsible for managing the
    /// status of the check and keeping the Catalog in sync.
    pub async fn agent_check_register(&self, req: structs::CheckDefinition) -> Result<()> {
        self.execute_request(Method::PUT, "/agent/check/register",  &(), &req).await
    }

    /// Deregister Check
    /// This endpoint remove a check from the local agent. The agent will take care of
    /// deregistering the check from the catalog. If the check with the provided ID
    /// does not exist, no action is taken.
    pub async fn deregister_check(&self, req: DeregisterCheckRequest) -> Result<()> {
        let path = format!("/agent/check/deregister/{}", req.check_id);
        self.execute_request(Method::PUT, &path,  &req, &()).await
    }

    /// TTL Check Pass
    /// This endpoint is used with a TTL type check to set the status of the check
    /// to passing and to reset the TTL clock.
    pub async fn check_pass(&self, req: AgentTTLCheckRequest) -> Result<()> {
        let path = format!("/agent/check/pass/{}", req.check_id);
        self.execute_request(Method::PUT, &path,  &req, &()).await
    }

    /// TTL Check Warn
    /// This endpoint is used with a TTL type check to set the status of the check
    /// to warning and to reset the TTL clock.
    pub async fn check_warn(&self, req: AgentTTLCheckRequest) -> Result<()> {
        let path = format!("/agent/check/warn/{}", req.check_id);
        self.execute_request(Method::PUT, &path,  &req, &()).await
    }

    /// TTL Check Fail
    /// This endpoint is used with a TTL type check to set the status of the check
    /// to critical and to reset the TTL clock.
    pub async fn check_fail(&self, req: AgentTTLCheckRequest) -> Result<()> {
        let path = format!("/agent/check/fail/{}", req.check_id);
        self.execute_request(Method::PUT, &path,  &req, &()).await
    }

    /// TTL Check Update
    /// This endpoint is used with a TTL type check to set the status of the check
    /// and to reset the TTL clock.
    pub async fn check_update(&self, req: AgentTTLCheckUpdateRequest) -> Result<()> {
        let path = format!("/agent/check/update/{}", req.check_id);
        let body = AgentTTLCheckUpdateRequestBody {
            status: req.status.clone(),
            output: req.output.clone(),
        };
        self.execute_request(Method::PUT, &path,  &req, &body).await
    }

    /// List Services
    /// This endpoint returns all the services that are registered with the local agent.
    /// These services were either provided through configuration files or added
    /// dynamically using the HTTP API.
    pub async fn list_services(&self, req: FilterRequest) -> Result<HashMap<String, structs::NodeService>> {
        self.execute_request(Method::GET, "/agent/services",  &req, &()).await
    }

    pub async fn get_service_configuration(&self, req: ServiceRequest) -> Result<structs::NodeService> {
        let path = format!("/agent/service/{}", req.service_id);
        self.execute_request(Method::GET, &path,  &req, &()).await
    }

    async fn execute_request<Q,B,T>(&self, method: Method, path: &str, query: &Q, body: &B) -> Result<T>
    where
        Q: Serialize,
        B: Serialize,
        T: DeserializeOwned,
    {
        let path = format!("{}{}{}", self.cfg.address, self.prefix, path);
        let mut b = self.requester.request(method, path);
        b = b.query(query);
        b = b.json(body);

        let resp = b.send().await?;
        resp.json().await.map_err(|e| anyhow!(e))
    }
}

#[inline]
fn read_env(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}