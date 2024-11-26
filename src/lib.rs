#![warn(missing_docs)]

use anyhow::{anyhow, Result};
use reqwest::{
    header::{HeaderMap, HeaderValue, AUTHORIZATION},
    Client, ClientBuilder, Method,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::collections::HashMap;

mod structs;

pub use reqwest::header;
pub use reqwest::Proxy;
pub use structs::*;

pub struct Config {
    pub token: String,
    pub address: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            token: read_env_or_default("CONSUL_TOKEN", ""),
            address: read_env_or_default("CONSUL_ADDRESS", "http://127.0.0.1:8500"),
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
        let mut headers = HeaderMap::new();
        if !self.cfg.token.is_empty() {
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", self.cfg.token)).unwrap(),
            );
        }

        let mut builder = ClientBuilder::new();
        builder = builder.default_headers(headers);

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

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct FilterRequestQuery {
    pub filter: Option<String>,

    #[cfg(feature = "enterprise")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ns: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DeregisterCheckRequestQuery {
    #[serde(skip_serializing)]
    pub check_id: String,

    #[cfg(feature = "enterprise")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ns: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AgentTTLCheckRequestQuery {
    #[serde(skip_serializing)]
    pub check_id: String,

    pub note: Option<String>,

    #[cfg(feature = "enterprise")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ns: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AgentTTLCheckUpdateRequestQuery {
    #[serde(skip_serializing)]
    pub check_id: String,

    #[cfg(feature = "enterprise")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ns: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AgentTTLCheckUpdateRequestBody {
    #[serde(rename = "Status")]
    pub status: Option<String>,

    #[serde(rename = "Output")]
    pub output: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ServiceConfigurationRequestQuery {
    #[serde(skip_serializing)]
    pub service_id: String,

    #[cfg(feature = "enterprise")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ns: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct LocalServiceHealthByNameRequestQuery {
    #[serde(skip_serializing)]
    pub service_name: String,

    #[cfg(feature = "enterprise")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ns: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct LocalServiceHealthByIDRequestQuery {
    #[serde(skip_serializing)]
    pub service_id: String,

    #[cfg(feature = "enterprise")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ns: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct RegisterServiceRequestQuery {
    /// Missing health checks from the request will be deleted from the agent.
    /// Using this parameter allows to idempotently register a service and
    /// its checks without having to manually deregister checks.
    #[serde(rename = "replace-existing-checks")]
    pub replace_existing_checks: String,

    #[cfg(feature = "enterprise")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ns: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DeregisterServiceRequestQuery {
    #[serde(skip_serializing)]
    pub service_id: String,

    #[cfg(feature = "enterprise")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ns: Option<String>,

    #[cfg(feature = "enterprise")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub partition: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct EnableMaintenanceModeRequestQuery {
    #[serde(skip_serializing)]
    pub service_id: String,

    /// Specifies whether to enable or disable maintenance mode.
    /// This is specified as part of the URL as a query string parameter.
    pub enable: bool,

    /// Specifies a text string explaining the reason for placing the node
    /// into maintenance mode. This is simply to aid human operators. If no
    /// reason is provided, a default value is used instead. This parameter
    /// must be URI-encoded.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,

    #[cfg(feature = "enterprise")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ns: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ConnectAuthorizeRequestQuery {
    #[cfg(feature = "enterprise")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ns: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ConnectAuthorizeRequestReply {
    /// True if authorized, false if not
    #[serde(rename = "Authorized")]
    pub authorized: bool,

    /// Reason for the Authorized value (whether true or false)
    #[serde(rename = "Reason")]
    pub reason: String,
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
    pub async fn checks(
        &self,
        q: &FilterRequestQuery,
    ) -> Result<HashMap<String, structs::HealthCheck>> {
        self.execute_request(Method::GET, "/agent/checks", q, &())
            .await
    }

    /// Register Check
    /// This endpoint adds a new check to the local agent. Checks may be of script,
    /// HTTP, TCP, UDP, or TTL type. The agent is responsible for managing the
    /// status of the check and keeping the Catalog in sync.
    pub async fn check_register(&self, b: &structs::CheckDefinition) -> Result<()> {
        self.execute_request(Method::PUT, "/agent/check/register", &(), b)
            .await
    }

    /// Deregister Check
    /// This endpoint remove a check from the local agent. The agent will take care of
    /// deregistering the check from the catalog. If the check with the provided ID
    /// does not exist, no action is taken.
    pub async fn deregister_check(&self, q: &DeregisterCheckRequestQuery) -> Result<()> {
        let path = format!("/agent/check/deregister/{}", q.check_id);
        self.execute_request(Method::PUT, &path, q, &()).await
    }

    /// TTL Check Pass
    /// This endpoint is used with a TTL type check to set the status of the check
    /// to passing and to reset the TTL clock.
    pub async fn check_pass(&self, q: &AgentTTLCheckRequestQuery) -> Result<()> {
        let path = format!("/agent/check/pass/{}", q.check_id);
        self.execute_request(Method::PUT, &path, q, &()).await
    }

    /// TTL Check Warn
    /// This endpoint is used with a TTL type check to set the status of the check
    /// to warning and to reset the TTL clock.
    pub async fn check_warn(&self, q: &AgentTTLCheckRequestQuery) -> Result<()> {
        let path = format!("/agent/check/warn/{}", q.check_id);
        self.execute_request(Method::PUT, &path, q, &()).await
    }

    /// TTL Check Fail
    /// This endpoint is used with a TTL type check to set the status of the check
    /// to critical and to reset the TTL clock.
    pub async fn check_fail(&self, q: &AgentTTLCheckRequestQuery) -> Result<()> {
        let path = format!("/agent/check/fail/{}", q.check_id);
        self.execute_request(Method::PUT, &path, q, &()).await
    }

    /// TTL Check Update
    /// This endpoint is used with a TTL type check to set the status of the check
    /// and to reset the TTL clock.
    pub async fn check_update(
        &self,
        q: &AgentTTLCheckUpdateRequestQuery,
        b: &AgentTTLCheckUpdateRequestBody,
    ) -> Result<()> {
        let path = format!("/agent/check/update/{}", q.check_id);
        self.execute_request(Method::PUT, &path, q, b).await
    }

    /// List Services
    /// This endpoint returns all the services that are registered with the local agent.
    /// These services were either provided through configuration files or added
    /// dynamically using the HTTP API.
    pub async fn list_services(
        &self,
        q: &FilterRequestQuery,
    ) -> Result<HashMap<String, structs::NodeService>> {
        self.execute_request(Method::GET, "/agent/services", q, &())
            .await
    }

    pub async fn service_configuration(
        &self,
        q: &ServiceConfigurationRequestQuery,
    ) -> Result<structs::NodeService> {
        let path = format!("/agent/service/{}", q.service_id);
        self.execute_request(Method::GET, &path, q, &()).await
    }

    /// Get local service health
    /// Retrieve an aggregated state of service(s) on the local agent by name.
    ///
    /// This endpoints support JSON format and text/plain formats, JSON
    /// being the default. In order to get the text format, you can
    /// append ?format=text to the URL or use Mime Content negotiation
    /// by specifying a HTTP Header Accept starting with text/plain.
    pub async fn local_service_health_by_name(
        &self,
        q: &LocalServiceHealthByNameRequestQuery,
    ) -> Result<Vec<structs::NodeService>> {
        let path = format!("/agent/health/service/name/{}", q.service_name);
        self.execute_request(Method::GET, &path, q, &()).await
    }

    /// Get local service health by ID
    /// Retrieve the health state of a specific service on the local agent
    /// by ID.
    pub async fn local_service_health_by_id(
        &self,
        q: &LocalServiceHealthByIDRequestQuery,
    ) -> Result<structs::NodeService> {
        let path = format!("/agent/health/service/id/{}", q.service_id);
        self.execute_request(Method::GET, &path, q, &()).await
    }

    /// Register Service
    /// This endpoint adds a new service, with optional health checks, to the
    /// local agent.
    ///
    /// The agent is responsible for managing the status of its local services, and
    /// for sending updates about its local services to the servers to keep the
    /// global catalog in sync.
    pub async fn register_service(
        &self,
        q: &RegisterServiceRequestQuery,
        b: &structs::ServiceDefinition,
    ) -> Result<structs::NodeService> {
        self.execute_request(Method::PUT, "/agent/service/register", q, b)
            .await
    }

    /// Deregister Service
    /// This endpoint removes a service from the local agent. If the service
    /// does not exist, no action is taken.
    ///
    /// The agent will take care of deregistering the service with the catalog.
    /// If there is an associated check, that is also deregistered.
    pub async fn deregister_service(&self, q: &DeregisterServiceRequestQuery) -> Result<()> {
        let path = format!("/agent/service/deregister/{}", q.service_id);
        self.execute_request(Method::PUT, &path, q, &()).await
    }

    /// Enable Maintenance Mode
    ///
    /// This endpoint places a given service into "maintenance mode". During
    /// maintenance mode, the service will be marked as unavailable and will
    /// not be present in DNS or API queries. This API call is idempotent.
    /// Maintenance mode is persistent and will be automatically restored on
    /// agent restart.
    pub async fn enable_maintenance_mode(
        &self,
        q: &EnableMaintenanceModeRequestQuery,
    ) -> Result<structs::NodeService> {
        let path = format!("/agent/service/maintenance/{}", q.service_id);
        self.execute_request(Method::PUT, &path, q, &()).await
    }

    pub async fn connect_authorize(
        &self,
        q: &ConnectAuthorizeRequestQuery,
        b: &structs::ConnectAuthorizeRequest,
    ) -> Result<ConnectAuthorizeRequestReply> {
        self.execute_request(Method::POST, "/agent/connect/authorize", q, b)
            .await
    }

    async fn execute_request<Q, B, T>(
        &self,
        method: Method,
        path: &str,
        query: &Q,
        body: &B,
    ) -> Result<T>
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
fn read_env_or_default(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}
