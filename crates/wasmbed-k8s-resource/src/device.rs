// SPDX-License-Identifier: AGPL-3.0
// Copyright © 2025 Wasmbed contributors

use chrono::{DateTime, Utc};
use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use wasmbed_types::GatewayReference;

#[derive(
    Clone,
    Debug,
    Eq,
    PartialEq,
    Serialize,
    Deserialize,
    JsonSchema,
    CustomResource,
)]
#[kube(
    namespaced,
    group = "wasmbed.github.io",
    version = "v0",
    kind = "Device",
    status = "DeviceStatus",
    selectable = ".spec.publicKey"
)]
#[serde(rename_all = "camelCase")]
pub struct DeviceSpec {
    pub public_key: String,
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
pub struct DeviceStatus {
    /// Current device phase
    #[serde(default)]
    pub phase: DevicePhase,

    /// Gateway pod name the device is connected to
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gateway: Option<GatewayReference>,

    /// Connection establishment timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connected_since: Option<DateTime<Utc>>,

    /// Last heartbeat timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_heartbeat: Option<DateTime<Utc>>,

    /// Pairing mode status
    #[serde(default)]
    pub pairing_mode: bool,
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema, Default, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub enum DevicePhase {
    #[default]
    Pending,
    /// Device is attempting to enroll
    Enrolling,
    /// Device has successfully enrolled but not yet connected
    Enrolled,
    /// Device is connected and active
    Connected,
    /// Device is disconnected but may reconnect
    Disconnected,
    /// Device is unreachable (heartbeat timeout)
    Unreachable,
}

impl DevicePhase {
    /// Validate if a transition from one phase to another is allowed
    pub fn validate_transition(from: DevicePhase, to: DevicePhase) -> bool {
        match (from, to) {
            // Valid transitions
            (DevicePhase::Pending, DevicePhase::Enrolling) => true,
            (DevicePhase::Enrolling, DevicePhase::Enrolled) => true,
            (DevicePhase::Enrolling, DevicePhase::Pending) => true, // Failed enrollment
            (DevicePhase::Enrolled, DevicePhase::Connected) => true,
            (DevicePhase::Connected, DevicePhase::Disconnected) => true,
            (DevicePhase::Disconnected, DevicePhase::Connected) => true,
            (DevicePhase::Connected, DevicePhase::Unreachable) => true,
            (DevicePhase::Disconnected, DevicePhase::Unreachable) => true,
            (DevicePhase::Unreachable, DevicePhase::Connected) => true,
            (DevicePhase::Unreachable, DevicePhase::Disconnected) => true,
            
            // Invalid transitions
            _ => false,
        }
    }
}
