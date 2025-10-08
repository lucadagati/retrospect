// SPDX-License-Identifier: AGPL-3.0
// Copyright © 2025 Wasmbed contributors

use crate::Application;

#[cfg(feature = "client")]
impl Application {
    /// Find applications by device name
    pub async fn find_by_device(
        api: kube::Api<Self>,
        device_name: &str,
    ) -> Result<Vec<Self>, kube::Error> {
        let apps = api.list(&kube::api::ListParams::default()).await?;
        
        Ok(apps
            .into_iter()
            .filter(|app| app.targets_device(device_name))
            .collect())
    }
}
