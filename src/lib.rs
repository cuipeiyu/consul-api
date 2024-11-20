use anyhow::Result;

pub mod structs;

pub struct AgentBuilder {
    prefix: String,
}

impl AgentBuilder {
    pub fn new() -> AgentBuilder {
        AgentBuilder {
            #[cfg(feature = "api-v1")]
            prefix: "/v1".to_string(),
        }
    }

    pub fn build(self) -> Result<AgentClient> {
        Ok(AgentClient {})
    }
}

pub struct AgentClient {}

impl AgentClient {
    pub fn acl_bootstrap_put(&self) -> Result<()> {
        Ok(())
    }
}
