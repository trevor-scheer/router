use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tower::BoxError;

/// The full identifier for an operation in a PQ list consists of an operation
/// ID and an optional client name.
#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct FullPersistedQueryOperationId {
    /// The operation ID (usually a hash).
    pub operation_id: String,
    /// The client name associated with the operation; if None, can be any client.
    pub client_name: Option<String>,
}

/// A single operation containing an ID and a body,
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestOperation {
    pub id: String,
    pub body: String,
    pub client_name: Option<String>,
}

/// The format of each persisted query chunk returned from uplink.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct SignedUrlChunk {
    pub(crate) format: String,
    pub(crate) version: u64,
    pub(crate) operations: Vec<ManifestOperation>,
}

impl SignedUrlChunk {
    pub(crate) fn validate(self) -> Result<Self, BoxError> {
        if self.format != "apollo-persisted-query-manifest" {
            return Err("chunk format is not 'apollo-persisted-query-manifest'".into());
        }

        if self.version != 1 {
            return Err("persisted query manifest chunk version is not 1".into());
        }

        Ok(self)
    }
}

/// An in memory cache of persisted queries.
#[derive(Debug, Clone)]
pub struct PersistedQueryManifest {
    manifest: HashMap<FullPersistedQueryOperationId, String>,
}

impl PersistedQueryManifest {
    /// Create a new persisted query manifest.
    pub fn new() -> Self {
        Self {
            manifest: HashMap::new(),
        }
    }

    /// Insert a new operation into the manifest.
    pub fn insert(&mut self, operation_id: FullPersistedQueryOperationId, body: String) {
        self.manifest.insert(operation_id, body);
    }

    /// Get an operation from the manifest.
    pub fn get(&self, operation_id: &FullPersistedQueryOperationId) -> Option<&String> {
        self.manifest.get(operation_id)
    }

    /// Get the number of operations in the manifest.
    pub fn len(&self) -> usize {
        self.manifest.len()
    }

    /// Get an iterator over the values in the manifest.
    pub fn values(&self) -> impl Iterator<Item = &String> {
        self.manifest.values()
    }

    /// Drain the manifest and return an iterator over the operations.
    pub fn drain(&mut self) -> impl Iterator<Item = (FullPersistedQueryOperationId, String)> {
        self.manifest.drain()
    }

    /// Add a chunk to the manifest.
    pub(crate) fn add_chunk(&mut self, chunk: SignedUrlChunk) {
        for operation in chunk.operations {
            self.manifest.insert(
                FullPersistedQueryOperationId {
                    operation_id: operation.id,
                    client_name: operation.client_name,
                },
                operation.body,
            );
        }
    }
}

impl From<Vec<ManifestOperation>> for PersistedQueryManifest {
    fn from(operations: Vec<ManifestOperation>) -> Self {
        let mut manifest = PersistedQueryManifest::new();
        for operation in operations {
            manifest.insert(
                FullPersistedQueryOperationId {
                    operation_id: operation.id,
                    client_name: operation.client_name,
                },
                operation.body,
            );
        }
        manifest
    }
}
