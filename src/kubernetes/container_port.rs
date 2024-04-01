use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// ContainerPort represents a network port in a single container.
#[allow(clippy::doc_markdown)]
#[derive(JsonSchema, Deserialize, Serialize, Clone, Debug, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ContainerPort {
    /// Number of port to expose on the pod's IP address. This must be a valid port number, 0 \< x \< 65536.
    #[allow(clippy::struct_field_names)]
    pub container_port: i32,

    /// What host IP to bind the external port to.
    pub host_ip: Option<String>,

    /// Number of port to expose on the host. If specified, this must be a valid port number, 0 \< x \< 65536. If HostNetwork is specified, this must match ContainerPort. Most containers do not need this.
    pub host_port: Option<i32>,

    /// Protocol for port. Must be UDP, TCP, or SCTP. Defaults to "TCP".
    pub protocol: Option<String>,
}

impl ContainerPort {
    pub fn into_container_port(self, name: String) -> k8s_openapi::api::core::v1::ContainerPort {
        k8s_openapi::api::core::v1::ContainerPort {
            container_port: self.container_port,
            host_ip: self.host_ip,
            host_port: self.host_port,
            name: Some(name),
            protocol: self.protocol,
        }
    }
}
