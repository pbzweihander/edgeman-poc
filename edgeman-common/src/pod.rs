use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

const POD_TIMESTAMP_ANNOTATION_KEY: &str = "edgeman.pbzweihander.dev/timestamp";

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Pod(pub k8s_openapi::api::core::v1::Pod);

impl Pod {
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.0.metadata.name.is_none() {
            Err(anyhow::anyhow!("`.metadata.name` must be present"))
        } else if self
            .0
            .metadata
            .annotations
            .as_ref()
            .and_then(|m| m.get(POD_TIMESTAMP_ANNOTATION_KEY))
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .is_none()
        {
            Err(anyhow::anyhow!(
                "`.metadata.annotation.{}` must be present and valid RFC3339 string",
                POD_TIMESTAMP_ANNOTATION_KEY
            ))
        } else {
            Ok(())
        }
    }

    /// Should be called after `validate`, or might be panic
    pub fn name(&self) -> &str {
        self.0
            .metadata
            .name
            .as_deref()
            .expect("`.metadata.name` must be present")
    }

    /// Should be called after `validate`, or might be panic
    pub fn timestamp(&self) -> DateTime<Utc> {
        self.0
            .metadata
            .annotations
            .as_ref()
            .and_then(|m| m.get(POD_TIMESTAMP_ANNOTATION_KEY))
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .unwrap_or_else(|| {
                panic!(
                    "`.metadata.annotation.{}` must be present and valid RFC3339 string",
                    POD_TIMESTAMP_ANNOTATION_KEY
                )
            })
            .with_timezone(&Utc)
    }
}
