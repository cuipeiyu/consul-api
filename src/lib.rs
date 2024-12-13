//#![warn(missing_docs)]

use anyhow::{anyhow, Result};
use reqwest::{
    header::{HeaderMap, HeaderValue, AUTHORIZATION},
    Method, Response, StatusCode,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// #[doc(hidden)]
// pub use reqwest::header;
#[doc(hidden)]
pub use reqwest::Proxy;

#[cfg(all(feature = "v1", feature = "v1_20_x"))]
mod structs_1_20_x;
#[cfg(all(feature = "v1", feature = "v1_20_x"))]
pub use structs_1_20_x::*;

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
    #[cfg(feature = "enterprise")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ns: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct LocalServiceHealthByIDRequestQuery {
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
pub struct KVReadKeyQuery {
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
pub struct KVCreateOrUpdateKeyQuery {
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
pub struct KVDeleteKeyQuery {
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

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CatalogRegisterEntityQuery {
    #[cfg(feature = "enterprise")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ns: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CatalogDeregisterEntityQuery {
    #[cfg(feature = "enterprise")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ns: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CatalogListServicesQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dc: Option<String>,

    #[serde(rename = "node-meta")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_meta: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<String>,

    #[cfg(feature = "enterprise")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ns: Option<String>,

    #[cfg(feature = "enterprise")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub partition: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CatalogListNodesForServiceQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dc: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub near: Option<String>,

    #[serde(rename = "node-meta")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_meta: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<String>,

    #[cfg(feature = "enterprise")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ns: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CatalogNodeServicesQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dc: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<String>,

    #[cfg(feature = "enterprise")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ns: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CatalogGatewayServicesQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dc: Option<String>,

    #[cfg(feature = "enterprise")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ns: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct EventFireQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dc: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub node: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct EventListQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub node: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct HealthListNodesQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dc: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<String>,

    #[cfg(feature = "enterprise")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ns: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct HealthListServicesQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dc: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub near: Option<String>,

    #[serde(rename = "node-meta")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_meta: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<String>,

    #[cfg(feature = "enterprise")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ns: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct HealthListServiceInstancesQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dc: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub near: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,

    #[serde(rename = "node-meta")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_meta: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub passing: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub peer: Option<String>,

    #[cfg(feature = "enterprise")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ns: Option<String>,

    #[cfg(feature = "enterprise")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sg: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct HealthListStateQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dc: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub near: Option<String>,

    #[serde(rename = "node-meta")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_meta: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<String>,

    #[cfg(feature = "enterprise")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ns: Option<String>,
}

#[cfg(feature = "enterprise")]
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct NamespaceCreateBody {
    /// The namespace's name. This field must be a valid DNS hostname label.
    ///
    /// required
    ///
    #[serde(rename = "Name")]
    pub name: String,

    /// Free form namespaces description.
    #[serde(rename = "Description")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(rename = "ACLs")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acls: Option<NamespaceACLConfig>,

    #[serde(rename = "Meta")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<::std::collections::HashMap<String, String>>,

    #[serde(rename = "Partition")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub partition: Option<String>,
}

#[cfg(feature = "enterprise")]
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct NamespaceDetail {
    /// The namespace's name.
    #[serde(rename = "Name")]
    pub name: String,

    /// Free form namespaces description.
    #[serde(rename = "Description")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(rename = "ACLs")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acls: Option<NamespaceACLConfig>,

    #[serde(rename = "Meta")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<::std::collections::HashMap<String, String>>,

    #[serde(rename = "CreateIndex")]
    pub create_index: u64,

    #[serde(rename = "ModifyIndex")]
    pub modify_index: u64,
}

#[cfg(feature = "enterprise")]
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct NamespaceReadQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub partition: Option<String>,
}

#[cfg(feature = "enterprise")]
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct NamespaceUpdateBody {
    /// If specified, this field must be an exact match with the name path
    /// parameter.
    #[serde(rename = "Name")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Free form namespaces description.
    #[serde(rename = "Description")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(rename = "ACLs")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acls: Option<NamespaceACLConfig>,

    #[serde(rename = "Meta")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<::std::collections::HashMap<String, String>>,

    #[serde(rename = "Partition")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub partition: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct StatusQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dc: Option<String>,
}

/// The Consul API client.
pub struct Client {
    cfg: Config,
    http: reqwest::Client,
    prefix: String,
}

impl Client {
    /// Creates a new client with the default configuration.
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
    pub async fn agent_check_deregister(&self, q: &DeregisterCheckRequestQuery) -> Result<bool> {
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
    ) -> Result<Option<AgentService>> {
        let path = format!("/agent/service/{}", q.service_id);
        let resp = self
            .execute_request(Method::GET, &path, q, None, &())
            .await?;

        if resp.status() == StatusCode::NOT_FOUND {
            return Ok(None);
        }

        Ok(Some(resp.json().await.map_err(|e| anyhow!(e))?))
    }

    /// Get local service health
    /// Retrieve an aggregated state of service(s) on the local agent by name.
    ///
    /// This endpoints support JSON format and text/plain formats, JSON
    /// being the default. In order to get the text format, you can
    /// append ?format=text to the URL or use Mime Content negotiation
    /// by specifying a HTTP Header Accept starting with text/plain.
    pub async fn agent_get_service_health_by_name<S: Into<String>>(
        &self,
        service_name: S,
        q: &LocalServiceHealthByNameRequestQuery,
    ) -> Result<Vec<AgentServiceChecksInfo>> {
        let path = format!("/agent/health/service/name/{}", service_name.into());
        let resp = self
            .execute_request(Method::GET, &path, q, None, &())
            .await?;
        resp.json().await.map_err(|e| anyhow!(e))
    }

    /// Get local service health by ID
    /// Retrieve the health state of a specific service on the local agent
    /// by ID.
    pub async fn agent_get_service_health_by_id<S: Into<String>>(
        &self,
        service_id: S,
        q: &LocalServiceHealthByIDRequestQuery,
    ) -> Result<Option<AgentServiceChecksInfo>> {
        let path = format!("/agent/health/service/id/{}", service_id.into());
        let resp = self
            .execute_request(Method::GET, &path, q, None, &())
            .await?;

        if resp.status() == StatusCode::NOT_FOUND {
            return Ok(None);
        }

        Ok(Some(resp.json().await.map_err(|e| anyhow!(e))?))
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
    ) -> Result<bool> {
        let resp = self
            .execute_request(Method::PUT, "/agent/service/register", q, None, b)
            .await?;
        Ok(resp.status() == StatusCode::OK)
    }

    /// Deregister Service
    /// This endpoint removes a service from the local agent. If the service
    /// does not exist, no action is taken.
    ///
    /// The agent will take care of deregistering the service with the catalog.
    /// If there is an associated check, that is also deregistered.
    pub async fn agent_deregister_service<S: Into<String>>(
        &self,
        service_id: S,
        q: &DeregisterServiceRequestQuery,
    ) -> Result<bool> {
        let path = format!("/agent/service/deregister/{}", service_id.into());
        let resp = self
            .execute_request(Method::PUT, &path, q, None, &())
            .await?;
        Ok(resp.status() == StatusCode::OK)
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

    /// Catalog Register Entity
    /// This endpoint is a low-level mechanism for registering or updating
    /// entries in the catalog. It is usually preferable to instead use the
    /// agent endpoints for registration as they are simpler and perform
    /// anti-entropy.
    pub async fn catalog_register_entity(
        &self,
        q: &CatalogRegisterEntityQuery,
        b: &RegisterRequest,
    ) -> Result<bool> {
        let resp = self
            .execute_request(Method::PUT, "/catalog/register", q, None, b)
            .await?;

        Ok(resp.status() == StatusCode::OK)
    }

    /// Catalog Deregister Entity
    /// This endpoint is a low-level mechanism for directly removing entries
    /// from the Catalog. It is usually preferable to instead use the agent
    /// endpoints for deregistration as they are simpler and perform
    /// anti-entropy.
    pub async fn catalog_deregister_entity(
        &self,
        q: &CatalogDeregisterEntityQuery,
        b: &DeregisterRequest,
    ) -> Result<bool> {
        let resp = self
            .execute_request(Method::PUT, "/catalog/deregister", q, None, b)
            .await?;

        Ok(resp.status() == StatusCode::OK)
    }

    /// Catalog List Datacenters
    /// This endpoint returns the list of all known datacenters. The
    /// datacenters will be sorted in ascending order based on the estimated
    /// median round trip time from the server to the servers in that
    /// datacenter.
    pub async fn catalog_list_datacenters(&self) -> Result<Vec<String>> {
        let resp = self
            .execute_request(Method::GET, "/catalog/datacenters", &(), None, &())
            .await?;

        resp.json().await.map_err(|e| anyhow!(e))
    }

    /// Catalog List Nodes
    /// This endpoint and returns the nodes registered in a given datacenter.
    pub async fn catalog_list_nodes(&self) -> Result<Vec<Node>> {
        let resp = self
            .execute_request(Method::GET, "/catalog/nodes", &(), None, &())
            .await?;

        resp.json().await.map_err(|e| anyhow!(e))
    }

    /// Catalog List Services
    /// This endpoint returns the services registered in a given datacenter.
    pub async fn catalog_list_services(
        &self,
        q: &CatalogListServicesQuery,
    ) -> Result<::std::collections::HashMap<String, Vec<String>>> {
        let resp = self
            .execute_request(Method::GET, "/catalog/services", q, None, &())
            .await?;

        resp.json().await.map_err(|e| anyhow!(e))
    }

    /// Catalog List Nodes for Service
    /// This endpoint returns the nodes providing a service in a given
    /// datacenter.
    pub async fn catalog_list_nodes_for_service<S: Into<String>>(
        &self,
        service_name: S,
        q: &CatalogListNodesForServiceQuery,
    ) -> Result<Vec<ServiceNode>> {
        let p = format!("/catalog/service/{}", service_name.into());

        let resp = self.execute_request(Method::GET, &p, q, None, &()).await?;

        resp.json().await.map_err(|e| anyhow!(e))
    }

    /// List Nodes for Mesh-capable Service
    /// This endpoint returns the nodes providing a mesh-capable service in a
    /// given datacenter. This will include both proxies and native
    /// integrations. A service may register both mesh-capable and incapable
    /// services at the same time, so this endpoint may be used to filter only
    /// the mesh-capable endpoints.
    pub async fn catalog_list_nodes_for_mesh_capable_service<S: Into<String>>(
        &self,
        service: S,
        q: &CatalogListNodesForServiceQuery,
    ) -> Result<Vec<ServiceNode>> {
        let p = format!("/catalog/connect/{}", service.into());

        let resp = self.execute_request(Method::GET, &p, q, None, &()).await?;

        resp.json().await.map_err(|e| anyhow!(e))
    }

    /// Retrieve Map of Services for a Node
    /// This endpoint returns the node's registered services.
    pub async fn catalog_node_services<S: Into<String>>(
        &self,
        node_name: S,
        q: &CatalogNodeServicesQuery,
    ) -> Result<Option<NodeServices>> {
        let p = format!("/catalog/node/{}", node_name.into());

        let resp = self.execute_request(Method::GET, &p, q, None, &()).await?;

        if resp.status() == StatusCode::NOT_FOUND {
            return Ok(None);
        }

        resp.json().await.map_err(|e| anyhow!(e))
    }

    /// List Services for Gateway
    /// This endpoint returns the services associated with an ingress gateway
    /// or terminating gateway.
    pub async fn catalog_gateway_services<S: Into<String>>(
        &self,
        gateway: S,
        q: &CatalogGatewayServicesQuery,
    ) -> Result<Vec<GatewayService>> {
        let p = format!("/catalog/gateway-services/{}", gateway.into());

        let resp = self.execute_request(Method::GET, &p, q, None, &()).await?;

        resp.json().await.map_err(|e| anyhow!(e))
    }

    /// Fire Event
    /// This endpoint triggers a new user event.
    pub async fn event_fire<S: Into<String>>(
        &self,
        name: S,
        body: Option<Vec<u8>>,
        q: &EventFireQuery,
    ) -> Result<bool> {
        let p = format!("/event/fire/{}", name.into());

        let resp = self.execute_request(Method::PUT, &p, q, body, &()).await?;

        Ok(resp.status() == StatusCode::OK)
    }

    /// List Events
    /// This endpoint returns the most recent events (up to 256) known by the
    /// agent. As a consequence of how the event command works, each agent may
    /// have a different view of the events. Events are broadcast using the
    /// gossip protocol, so they have no global ordering nor do they make a
    /// promise of delivery.
    pub async fn event_list(&self, q: &EventListQuery) -> Result<Vec<UserEvent>> {
        let resp = self
            .execute_request(Method::GET, "/event/list", q, None, &())
            .await?;

        let mut list: Vec<UserEvent> = resp.json().await.map_err(|e| anyhow!(e))?;

        for item in list.iter_mut() {
            item.payload = item.payload.clone().map_or(None, |v| {
                // 'bnVsbA==' is null
                if v.0 == "bnVsbA==" {
                    None
                } else {
                    Some(v)
                }
            })
        }

        Ok(list)
    }

    /// List Checks for Node
    /// This endpoint returns the checks specific to the node provided on the
    /// path.
    pub async fn health_list_nodes<S: Into<String>>(
        &self,
        node: S,
        q: &HealthListNodesQuery,
    ) -> Result<Vec<HealthCheck>> {
        let p = format!("/health/node/{}", node.into());

        let resp = self.execute_request(Method::GET, &p, q, None, &()).await?;

        resp.json().await.map_err(|e| anyhow!(e))
    }

    /// List Checks for Service
    /// This endpoint returns the checks associated with the service provided
    /// on the path.
    pub async fn health_list_services<S: Into<String>>(
        &self,
        service: S,
        q: &HealthListServicesQuery,
    ) -> Result<Vec<HealthCheck>> {
        let p = format!("/health/checks/{}", service.into());

        let resp = self.execute_request(Method::GET, &p, q, None, &()).await?;

        resp.json().await.map_err(|e| anyhow!(e))
    }

    /// List Service Instances for Service
    /// This endpoint returns the service instances providing the service
    /// indicated on the path. Users can also build in support for dynamic load
    /// balancing and other features by incorporating the use of health checks.
    pub async fn health_list_service_instances<S: Into<String>>(
        &self,
        service: S,
        q: &HealthListServiceInstancesQuery,
    ) -> Result<Vec<CheckServiceNode>> {
        let p = format!("/health/service/{}", service.into());

        let resp = self.execute_request(Method::GET, &p, q, None, &()).await?;

        resp.json().await.map_err(|e| anyhow!(e))
    }

    /// List Service Instances for Mesh-enabled Service
    ///
    /// This endpoint returns the service instances providing a mesh-capable
    /// service in a given datacenter. This will include both proxies and
    /// native integrations. A service may register both mesh-capable and
    /// incapable services at the same time, so this endpoint may be used to
    /// filter only the mesh-capable endpoints.
    ///
    pub async fn health_list_service_instances_for_mesh_capable<S: Into<String>>(
        &self,
        service: S,
        q: &HealthListServiceInstancesQuery,
    ) -> Result<Vec<CheckServiceNode>> {
        let p = format!("/health/connect/{}", service.into());

        let resp = self.execute_request(Method::GET, &p, q, None, &()).await?;

        resp.json().await.map_err(|e| anyhow!(e))
    }

    /// List Service Instances for Ingress Gateways Associated with a Service
    ///
    /// This API is available in Consul versions 1.8.0 and later.
    ///
    /// This endpoint returns the service instances providing an ingress
    /// gateway for a service in a given datacenter.
    ///
    pub async fn health_list_service_instances_for_ingress_gateways<S: Into<String>>(
        &self,
        service: S,
        q: &HealthListServiceInstancesQuery,
    ) -> Result<Vec<CheckServiceNode>> {
        let p = format!("/health/ingress/{}", service.into());

        let resp = self.execute_request(Method::GET, &p, q, None, &()).await?;

        resp.json().await.map_err(|e| anyhow!(e))
    }

    /// List Checks in State
    ///
    /// This endpoint returns the checks in the state provided on the path.
    ///
    pub async fn health_list_state(
        &self,
        state: Health,
        q: &HealthListStateQuery,
    ) -> Result<Vec<HealthCheck>> {
        let p = format!("/health/state/{}", state);

        let resp = self.execute_request(Method::GET, &p, q, None, &()).await?;

        resp.json().await.map_err(|e| anyhow!(e))
    }

    /// Read Key
    /// This endpoint returns the specified key. If no key exists at the given
    /// path, a 404 is returned instead of a 200 response.
    pub async fn kv_read_key<S: Into<String>>(
        &self,
        key: S,
        q: &KVReadKeyQuery,
    ) -> Result<Option<Vec<u8>>> {
        let path = format!("/kv/{}", key.into());
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
    pub async fn kv_create_or_update_key<S: Into<String>>(
        &self,
        key: S,
        b: Vec<u8>,
        q: &KVCreateOrUpdateKeyQuery,
    ) -> Result<bool> {
        let path = format!("/kv/{}", key.into());
        let resp = self
            .execute_request(Method::PUT, &path, q, Some(b), &())
            .await?;
        resp.json().await.map_err(|e| anyhow!(e))
    }

    /// Delete Key
    /// This endpoint deletes a single key or all keys sharing a prefix.
    pub async fn kv_delete_key<S: Into<String>>(
        &self,
        key: S,
        q: &KVDeleteKeyQuery,
    ) -> Result<bool> {
        let path = format!("/kv/{}", key.into());
        let resp = self
            .execute_request(Method::DELETE, &path, q, None, &())
            .await?;
        resp.json().await.map_err(|e| anyhow!(e))
    }

    /// Create a Namespace
    ///
    /// This feature requires Consul Enterprise.
    ///
    /// This endpoint creates a new Namespace.
    ///
    #[cfg(feature = "enterprise")]
    pub async fn namespace_create(&self, b: &NamespaceCreateBody) -> Result<NamespaceDetail> {
        let resp = self
            .execute_request(Method::PUT, "/namespace", &(), None, b)
            .await?;
        resp.json().await.map_err(|e| anyhow!(e))
    }

    /// Read a Namespace
    ///
    /// This feature requires Consul Enterprise.
    ///
    /// This endpoint reads a Namespace with the given name.
    ///
    #[cfg(feature = "enterprise")]
    pub async fn namespace_read<S: Into<String>>(
        &self,
        name: S,
        q: &NamespaceReadQuery,
    ) -> Result<Option<NamespaceDetail>> {
        let p = format!("/namespace/{}", name.into());

        let resp = self.execute_request(Method::GET, &p, &q, None, &()).await?;

        if resp.status() == StatusCode::NOT_FOUND {
            return Ok(None);
        }

        Ok(Some(resp.json().await.map_err(|e| anyhow!(e))?))
    }

    /// Update a Namespace
    ///
    /// This feature requires Consul Enterprise.
    ///
    /// This endpoint reads a Namespace with the given name.
    ///
    #[cfg(feature = "enterprise")]
    pub async fn namespace_update<S: Into<String>>(
        &self,
        name: S,
        b: &NamespaceUpdateBody,
    ) -> Result<NamespaceDetail> {
        let p = format!("/namespace/{}", name.into());

        let resp = self.execute_request(Method::PUT, &p, &(), None, &b).await?;

        resp.json().await.map_err(|e| anyhow!(e))
    }

    /// Get Raft Leader
    ///
    /// This endpoint returns the Raft leader for the datacenter in which the
    /// agent is running.
    ///
    pub async fn status_leader(&self, q: &StatusQuery) -> Result<String> {
        let resp = self
            .execute_request(Method::GET, "/status/leader", q, None, &())
            .await?;

        resp.text_with_charset("utf-8")
            .await
            .map_err(|e| anyhow!(e))
    }

    /// List Raft Peers
    ///
    /// This endpoint retrieves the Raft peers for the datacenter in which the
    /// agent is running. This list of peers is strongly consistent and can be
    /// useful in determining when a given server has successfully joined the
    /// cluster.
    ///
    pub async fn status_peers(&self, q: &StatusQuery) -> Result<Vec<String>> {
        let resp = self
            .execute_request(Method::GET, "/status/peers", q, None, &())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_client() {
        let _ = Client::new();
    }
}
