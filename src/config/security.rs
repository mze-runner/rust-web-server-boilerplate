use serde::Deserialize;

// Import the Validatable trait from parent module
use super::Validatable;

#[derive(Debug, Clone, Deserialize)]
pub struct Security {
    #[serde(default = "default_cors_origins")]
    pub cors_allowed_origins: Vec<String>,
    #[serde(default)]
    pub trusted_proxies: Vec<String>,
    #[serde(default = "default_rate_limit")]
    pub rate_limit_requests_per_minute: u32,
}

impl Validatable for Security {
    fn validate(&self) -> anyhow::Result<()> {
        if self.rate_limit_requests_per_minute == 0 {
            anyhow::bail!("Rate limit cannot be 0");
        }

        if self.rate_limit_requests_per_minute > 10000 {
            anyhow::bail!("Rate limit too high (max 10000/min)");
        }

        Ok(())
    }
}

fn default_cors_origins() -> Vec<String> {
    vec!["http://localhost:3000".to_string()]
}

fn default_rate_limit() -> u32 {
    60
}
