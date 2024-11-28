//#![warn(missing_docs)]

use anyhow::{anyhow, Result};
use reqwest::{
    header::{HeaderMap, HeaderValue, AUTHORIZATION},
    Method, Response, StatusCode,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub use reqwest::header;
pub use reqwest::Proxy;

#[cfg(all(feature = "v1", feature = "v1_20_1"))]
mod structs_1_20_1;
#[cfg(all(feature = "v1", feature = "v1_20_1"))]
pub use structs_1_20_1::*;

pub struct Config {
    pub token: String,
    pub address: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            token: read_env_or_default("CONSUL_TOKEN", ""),
            address: read_env_or_default("CONSUL_ADDRESS", "http://127.0.0.1:8500"),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::from_env()
    }
}

pub struct ClientBuilder {
    cfg: Config,
    proxies: Vec<Proxy>,
}

impl ClientBuilder {
    pub fn new(cfg: Config) -> Self {
        Self {
            cfg,
            proxies: vec![],
        }
    }

    pub fn with_proxy(mut self, proxy: Proxy) -> Self {
        self.proxies.push(proxy);
        self
    }

    pub fn build(self) -> Result<Client> {
        let mut headers = HeaderMap::new();
        if !self.cfg.token.is_empty() {
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", self.cfg.token)).unwrap(),
            );
        }

        let mut builder = reqwest::ClientBuilder::new();
        builder = builder.default_headers(headers);

        for proxy in self.proxies {
            // add proxy
            builder = builder.proxy(proxy)
        }

        Ok(Client {
            cfg: self.cfg,
            http: builder.build()?,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replace_existing_checks: Option<String>,

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

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct KVReadKeyRequestQuery {
    /// Specifies the path of the key to read.
    #[serde(skip_serializing)]
    pub key: String,

    /// Specifies the datacenter to query. This will default to the
    /// datacenter of the agent being queried.
    pub dc: Option<String>,

    /// Specifies if the lookup should be recursive and treat key as a
    /// prefix instead of a literal match.
    pub recurse: Option<bool>,

    /// Specifies the response is just the raw value of the key, without
    /// any encoding or metadata.
    pub raw: Option<bool>,

    /// Specifies to return only keys (no values or metadata). Specifying
    /// this parameter implies recurse.
    pub keys: Option<bool>,

    /// Specifies the string to use as a separator for recursive key
    /// lookups. This option is only used when paired with the keys
    /// parameter to limit the prefix of keys returned, only up to the
    /// given separator.
    pub separator: Option<String>,

    #[cfg(feature = "enterprise")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ns: Option<String>,

    /// The admin partition to use. If not provided, the partition is
    /// inferred from the request's ACL token, or defaults to the default
    /// partition.
    #[cfg(feature = "enterprise")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub partition: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct KVCreateOrUpdateKeyRequestQuery {
    /// Specifies the path of the key to create or update.
    #[serde(skip_serializing)]
    pub key: String,

    /// Specifies the datacenter to query. This will default to the
    /// datacenter of the agent being queried.
    pub dc: Option<String>,

    /// Specifies an unsigned value between 0 and (2^64)-1 to store with
    /// the key. API consumers can use this field any way they choose for
    /// their application.
    pub flags: Option<u64>,

    /// Specifies to use a Check-And-Set operation. This is very useful as a
    /// building block for more complex synchronization primitives. If the
    /// index is 0, Consul will only put the key if it does not already exist.
    /// If the index is non-zero, the key is only set if the index matches the
    /// ModifyIndex of that key.
    pub cas: Option<u64>,

    /// Supply a session ID to use in a lock acquisition operation. This is
    /// useful as it allows leader election to be built on top of Consul. If
    /// the lock is not held and the session is valid, this increments the
    /// LockIndex and sets the Session value of the key in addition to updating
    /// the key contents. A key does not need to exist to be acquired. If the
    /// lock is already held by the given session, then the LockIndex is not
    /// incremented but the key contents are updated. This lets the current
    /// lock holder update the key contents without having to give up the lock
    /// and reacquire it. Note that an update that does not include the acquire
    /// parameter will proceed normally even if another session has locked the
    /// key.
    pub acquire: Option<String>,

    /// Supply a session ID to use in a release operation. This is useful when
    /// paired with ?acquire= as it allows clients to yield a lock. This will
    /// leave the LockIndex unmodified but will clear the associated Session of
    /// the key. The key must be held by this session to be unlocked.
    pub release: Option<String>,

    #[cfg(feature = "enterprise")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ns: Option<String>,

    #[cfg(feature = "enterprise")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub partition: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct KVDeleteKeyRequestQuery {
    /// Specifies the path of the key to delete.
    #[serde(skip_serializing)]
    pub key: String,

    /// Specifies the datacenter to query. This will default to the datacenter
    /// of the agent being queried. If the DC is invalid, the error "No path to
    /// datacenter" is returned.
    pub dc: Option<String>,

    /// Specifies to delete all keys which have the specified prefix. Without
    /// this, only a key with an exact match will be deleted.
    pub recurse: Option<bool>,

    /// Specifies to use a Check-And-Set operation. This is very useful as a
    /// building block for more complex synchronization primitives. Unlike PUT,
    /// the index must be greater than 0 for Consul to take any action: a 0
    /// index will not delete the key. If the index is non-zero, the key is
    /// only deleted if the index matches the ModifyIndex of that key.
    pub cas: Option<u64>,

    #[cfg(feature = "enterprise")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ns: Option<String>,

    #[cfg(feature = "enterprise")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub partition: Option<String>,
}

pub struct Client {
    cfg: Config,
    http: reqwest::Client,
    prefix: String,
}

impl Client {
    pub fn new() -> Self {
        ClientBuilder::new(Config::default()).build().unwrap()
    }

    /// List Checks
    /// This endpoint returns all checks that are registered with the local agent.
    /// These checks were either provided through configuration files or added
    /// dynamically using the HTTP API.
    pub async fn agent_checks(
        &self,
        q: &FilterRequestQuery,
    ) -> Result<HashMap<String, HealthCheck>> {
        let resp = self
            .execute_request(Method::GET, "/agent/checks", q, None, &())
            .await?;

        resp.json().await.map_err(|e| anyhow!(e))
    }

    /// Register Check
    /// This endpoint adds a new check to the local agent. Checks may be of script,
    /// HTTP, TCP, UDP, or TTL type. The agent is responsible for managing the
    /// status of the check and keeping the Catalog in sync.
    pub async fn agent_check_register(&self, b: &CheckDefinition) -> Result<bool> {
        let resp = self
            .execute_request(Method::PUT, "/agent/check/register", &(), None, b)
            .await?;
        Ok(resp.status() == StatusCode::OK)
    }

    /// Deregister Check
    /// This endpoint remove a check from the local agent. The agent will take care of
    /// deregistering the check from the catalog. If the check with the provided ID
    /// does not exist, no action is taken.
    pub async fn agent_deregister_check(&self, q: &DeregisterCheckRequestQuery) -> Result<bool> {
        let path = format!("/agent/check/deregister/{}", q.check_id);
        let resp = self
            .execute_request(Method::PUT, &path, q, None, &())
            .await?;
        Ok(resp.status() == StatusCode::OK)
    }

    /// TTL Check Pass
    /// This endpoint is used with a TTL type check to set the status of the check
    /// to passing and to reset the TTL clock.
    pub async fn agent_check_pass(&self, q: &AgentTTLCheckRequestQuery) -> Result<bool> {
        let path = format!("/agent/check/pass/{}", q.check_id);
        let resp = self
            .execute_request(Method::PUT, &path, q, None, &())
            .await?;

        Ok(resp.status() == StatusCode::OK)
    }

    /// TTL Check Warn
    /// This endpoint is used with a TTL type check to set the status of the check
    /// to warning and to reset the TTL clock.
    pub async fn agent_check_warn(&self, q: &AgentTTLCheckRequestQuery) -> Result<()> {
        let path = format!("/agent/check/warn/{}", q.check_id);
        let resp = self
            .execute_request(Method::PUT, &path, q, None, &())
            .await?;
        resp.json().await.map_err(|e| anyhow!(e))
    }

    /// TTL Check Fail
    /// This endpoint is used with a TTL type check to set the status of the check
    /// to critical and to reset the TTL clock.
    pub async fn agent_check_fail(&self, q: &AgentTTLCheckRequestQuery) -> Result<bool> {
        let path = format!("/agent/check/fail/{}", q.check_id);
        let resp = self
            .execute_request(Method::PUT, &path, q, None, &())
            .await?;
        Ok(resp.status() == StatusCode::OK)
    }

    /// TTL Check Update
    /// This endpoint is used with a TTL type check to set the status of the check
    /// and to reset the TTL clock.
    pub async fn agent_check_update(
        &self,
        q: &AgentTTLCheckUpdateRequestQuery,
        b: &AgentTTLCheckUpdateRequestBody,
    ) -> Result<bool> {
        let path = format!("/agent/check/update/{}", q.check_id);
        let resp = self.execute_request(Method::PUT, &path, q, None, b).await?;
        Ok(resp.status() == StatusCode::OK)
    }

    /// List Services
    /// This endpoint returns all the services that are registered with the local agent.
    /// These services were either provided through configuration files or added
    /// dynamically using the HTTP API.
    pub async fn agent_services(
        &self,
        q: &FilterRequestQuery,
    ) -> Result<HashMap<String, AgentService>> {
        let resp = self
            .execute_request(Method::GET, "/agent/services", q, None, &())
            .await?;
        resp.json().await.map_err(|e| anyhow!(e))
    }

    pub async fn agent_service_configuration(
        &self,
        q: &ServiceConfigurationRequestQuery,
    ) -> Result<AgentService> {
        let path = format!("/agent/service/{}", q.service_id);
        let resp = self
            .execute_request(Method::GET, &path, q, None, &())
            .await?;
        resp.json().await.map_err(|e| anyhow!(e))
    }

    /// Get local service health
    /// Retrieve an aggregated state of service(s) on the local agent by name.
    ///
    /// This endpoints support JSON format and text/plain formats, JSON
    /// being the default. In order to get the text format, you can
    /// append ?format=text to the URL or use Mime Content negotiation
    /// by specifying a HTTP Header Accept starting with text/plain.
    pub async fn agent_get_service_health_by_name(
        &self,
        q: &LocalServiceHealthByNameRequestQuery,
    ) -> Result<Vec<AgentServiceChecksInfo>> {
        let path = format!("/agent/health/service/name/{}", q.service_name);
        let resp = self
            .execute_request(Method::GET, &path, q, None, &())
            .await?;
        resp.json().await.map_err(|e| anyhow!(e))
    }

    /// Get local service health by ID
    /// Retrieve the health state of a specific service on the local agent
    /// by ID.
    pub async fn agent_get_service_health_by_id(
        &self,
        q: &LocalServiceHealthByIDRequestQuery,
    ) -> Result<AgentServiceChecksInfo> {
        let path = format!("/agent/health/service/id/{}", q.service_id);
        let resp = self
            .execute_request(Method::GET, &path, q, None, &())
            .await?;
        resp.json().await.map_err(|e| anyhow!(e))
    }

    /// Register Service
    /// This endpoint adds a new service, with optional health checks, to the
    /// local agent.
    ///
    /// The agent is responsible for managing the status of its local services, and
    /// for sending updates about its local services to the servers to keep the
    /// global catalog in sync.
    pub async fn agent_register_service(
        &self,
        q: &RegisterServiceRequestQuery,
        b: &ServiceDefinition,
    ) -> Result<()> {
        let resp = self
            .execute_request(Method::PUT, "/agent/service/register", q, None, b)
            .await?;
        resp.json().await.map_err(|e| anyhow!(e))
    }

    /// Deregister Service
    /// This endpoint removes a service from the local agent. If the service
    /// does not exist, no action is taken.
    ///
    /// The agent will take care of deregistering the service with the catalog.
    /// If there is an associated check, that is also deregistered.
    pub async fn agent_deregister_service(&self, q: &DeregisterServiceRequestQuery) -> Result<()> {
        let path = format!("/agent/service/deregister/{}", q.service_id);
        let resp = self
            .execute_request(Method::PUT, &path, q, None, &())
            .await?;
        resp.json().await.map_err(|e| anyhow!(e))
    }

    /// Enable Maintenance Mode
    ///
    /// This endpoint places a given service into "maintenance mode". During
    /// maintenance mode, the service will be marked as unavailable and will
    /// not be present in DNS or API queries. This API call is idempotent.
    /// Maintenance mode is persistent and will be automatically restored on
    /// agent restart.
    pub async fn agent_enable_maintenance_mode(
        &self,
        q: &EnableMaintenanceModeRequestQuery,
    ) -> Result<bool> {
        let path = format!("/agent/service/maintenance/{}", q.service_id);
        let resp = self
            .execute_request(Method::PUT, &path, q, None, &())
            .await?;
        Ok(resp.status() == StatusCode::OK)
    }

    pub async fn agent_connect_authorize(
        &self,
        q: &ConnectAuthorizeRequestQuery,
        b: &ConnectAuthorizeRequest,
    ) -> Result<ConnectAuthorizeRequestReply> {
        let resp = self
            .execute_request(Method::POST, "/agent/connect/authorize", q, None, b)
            .await?;
        resp.json().await.map_err(|e| anyhow!(e))
    }

    /// Read Key
    /// This endpoint returns the specified key. If no key exists at the given
    /// path, a 404 is returned instead of a 200 response.
    pub async fn kv_read_key(&self, q: &KVReadKeyRequestQuery) -> Result<Option<Vec<u8>>> {
        let path = format!("/kv/{}", q.key);
        let resp = self
            .execute_request(Method::GET, &path, q, None, &())
            .await?;

        if resp.status() == StatusCode::NOT_FOUND {
            return Ok(None);
        }

        let full = resp.bytes().await?;

        if full.is_empty() {
            return Ok(Some(vec![]));
        }

        Ok(Some(full.to_vec()))
    }

    /// Create/Update Key
    /// This endpoint updates the value of the specified key. If no key exists
    /// at the given path, the key will be created.
    pub async fn kv_create_or_update_key(
        &self,
        q: &KVCreateOrUpdateKeyRequestQuery,
        b: Vec<u8>,
    ) -> Result<bool> {
        let path = format!("/kv/{}", q.key);
        let resp = self
            .execute_request(Method::PUT, &path, q, Some(b), &())
            .await?;
        resp.json().await.map_err(|e| anyhow!(e))
    }

    /// Delete Key
    /// This endpoint deletes a single key or all keys sharing a prefix.
    pub async fn kv_delete_key(&self, q: &KVDeleteKeyRequestQuery) -> Result<bool> {
        let path = format!("/kv/{}", q.key);
        let resp = self
            .execute_request(Method::DELETE, &path, q, None, &())
            .await?;
        resp.json().await.map_err(|e| anyhow!(e))
    }

    async fn execute_request<Q, B>(
        &self,
        method: Method,
        path: &str,
        query: &Q,
        raw_body: Option<Vec<u8>>,
        json_body: &B,
    ) -> Result<Response>
    where
        Q: Serialize,
        B: Serialize,
    {
        let path = format!("{}{}{}", self.cfg.address, self.prefix, path);
        let mut b = self.http.request(method.clone(), &path);

        b = b.query(query);

        if method == Method::PUT || method == Method::POST {
            if let Some(body) = raw_body {
                b = b.body(body)
            } else {
                b = b.json(json_body);
            }
        }

        let resp = b.send().await?;
        Ok(resp)
    }
}

#[inline]
fn read_env_or_default(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}
