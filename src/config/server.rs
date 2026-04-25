use serde::Deserialize;

use super::Validatable;

/// Server configuration
#[derive(Debug, Clone, Deserialize)]
pub struct Server {
    /// Server bind address, e.g. "0.0.0.0"
    pub addr: String,
    /// Server port, e.g. 8080
    pub port: u16,
    /// Maximum request body size in bytes (default: 1 MiB)
    pub request_body_limit_bytes: usize,
}

impl Validatable for Server {
    fn validate(&self) -> anyhow::Result<()> {
        // Validate port (though u16 already constrains this, we check for 0)
        if self.port == 0 {
            anyhow::bail!("Server port cannot be 0");
        }

        // Validate body limit (max 100MB is reasonable)
        const MAX_BODY_SIZE: usize = 100 * 1024 * 1024;
        if self.request_body_limit_bytes > MAX_BODY_SIZE {
            anyhow::bail!(
                "Request body limit too large: {} bytes (max {})",
                self.request_body_limit_bytes,
                MAX_BODY_SIZE
            );
        }

        if self.request_body_limit_bytes == 0 {
            anyhow::bail!("Request body limit cannot be 0");
        }

        Ok(())
    }
}
