// WASM runtime module
use heapless::{String, Vec};
use log::info;
use core::str::FromStr;

#[derive(Debug, Clone, PartialEq)]
pub enum ApplicationStatus {
    Loading,
    Running,
    Stopped,
    Error(String<64>),
}

pub struct WasmApplication {
    id: String<32>,
    bytecode: Vec<u8, 1024>,
    status: ApplicationStatus,
    memory_usage: usize,
}

pub struct WasmRuntime {
    applications: Vec<WasmApplication, 4>,
    initialized: bool,
}

impl WasmRuntime {
    pub fn new() -> Self {
        Self {
            applications: Vec::new(),
            initialized: false,
        }
    }

    pub fn init(&mut self) -> Result<(), &'static str> {
        info!("Initializing WASM runtime...");
        // Simulate WASM runtime initialization
        self.initialized = true;
        info!("WASM runtime initialized.");
        Ok(())
    }

    pub fn deploy_application(&mut self, app_id: &str, bytecode: &[u8]) -> Result<(), &'static str> {
        if !self.initialized {
            return Err("WASM runtime not initialized");
        }

        info!("Deploying WASM application: {}", app_id);

        let id = String::from_str(app_id).map_err(|_| "App ID too long")?;
        let mut app_bytecode = Vec::new();
        app_bytecode.extend_from_slice(bytecode).map_err(|_| "Bytecode too large")?;

        let app = WasmApplication {
            id,
            bytecode: app_bytecode,
            status: ApplicationStatus::Loading,
            memory_usage: 0,
        };

        self.applications.push(app).map_err(|_| "Too many applications")?;
        info!("WASM application {} deployed.", app_id);

        // Simulate starting the application
        if let Some(app) = self.applications.iter_mut().find(|a| a.id == app_id) {
            app.status = ApplicationStatus::Running;
            info!("WASM application {} started.", app_id);
        }

        Ok(())
    }

    pub fn stop_application(&mut self, app_id: &str) -> Result<(), &'static str> {
        if !self.initialized {
            return Err("WASM runtime not initialized");
        }

        info!("Stopping WASM application: {}", app_id);

        if let Some(app) = self.applications.iter_mut().find(|a| a.id == app_id) {
            app.status = ApplicationStatus::Stopped;
            info!("WASM application {} stopped.", app_id);
            Ok(())
        } else {
            Err("Application not found")
        }
    }

    pub fn run_applications(&mut self) -> Result<(), &'static str> {
        if !self.initialized {
            return Err("WASM runtime not initialized");
        }
        
        // Run all active applications
        let mut indices_to_run = Vec::<usize, 4>::new();
        
        // Collect indices of running applications
        for i in 0..self.applications.len() {
            if let Some(app) = self.applications.get(i) {
                if app.status == ApplicationStatus::Running {
                    indices_to_run.push(i).map_err(|_| "Too many applications")?;
                }
            }
        }
        
        // Execute applications
        for i in indices_to_run.iter() {
            if let Some(app) = self.applications.get_mut(*i) {
                // Simulate execution directly here to avoid borrow checker issues
                app.memory_usage = 1024 + (app.memory_usage % 1000);
            }
        }
        
        Ok(())
    }

    pub fn get_application_count(&self) -> usize {
        self.applications.len()
    }

    pub fn get_running_application_count(&self) -> usize {
        self.applications.iter()
            .filter(|app| app.status == ApplicationStatus::Running)
            .count()
    }

    pub fn get_total_memory_usage(&self) -> usize {
        self.applications.iter()
            .map(|app| app.memory_usage)
            .sum()
    }

    pub fn list_applications(&self) -> Vec<String<32>, 4> {
        self.applications.iter()
            .map(|app| app.id.clone())
            .collect()
    }
}
