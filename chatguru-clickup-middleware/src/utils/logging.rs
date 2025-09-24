use tracing::{info, warn, error, debug};

pub fn log_request_received(endpoint: &str, method: &str) {
    info!("Request received: {} {}", method, endpoint);
}

pub fn log_request_processed(endpoint: &str, status: u16, duration_ms: u64) {
    info!("Request processed: {} - Status: {} - Duration: {}ms", 
          endpoint, status, duration_ms);
}


pub fn log_clickup_task_created(task_id: &str, title: &str) {
    info!("ClickUp task created successfully: {} - Title: {}", task_id, title);
}

pub fn log_clickup_api_error(endpoint: &str, status: Option<u16>, error: &str) {
    error!("ClickUp API error: {} - Status: {:?} - Error: {}", endpoint, status, error);
}

#[allow(dead_code)]
pub fn log_pubsub_published(topic: &str, message_id: &str) {
    info!("Message published to Pub/Sub: Topic: {} - Message ID: {}", topic, message_id);
}

#[allow(dead_code)]
pub fn log_pubsub_error(topic: &str, error: &str) {
    error!("Pub/Sub error: Topic: {} - Error: {}", topic, error);
}

pub fn log_config_loaded(env: &str) {
    info!("Configuration loaded successfully for environment: {}", env);
}

pub fn log_server_startup(port: u16) {
    info!("🚀 ChatGuru-ClickUp middleware server starting on port {}", port);
}

pub fn log_server_ready(port: u16) {
    info!("✅ Server ready and listening on http://0.0.0.0:{}", port);
}

pub fn log_health_check() {
    debug!("Health check requested");
}

pub fn log_integration_status_check() {
    debug!("Integration status check requested");
}

pub fn log_validation_error(field: &str, message: &str) {
    warn!("Validation error: {} - {}", field, message);
}

pub fn log_clickup_task_updated(task_id: &str, title: &str) {
    info!("✏️ ClickUp task updated successfully: {} - Title: {}", task_id, title);
}

pub fn log_info(message: &str) {
    info!("{}", message);
}

pub fn log_error(message: &str) {
    error!("{}", message);
}

pub fn log_warning(message: &str) {
    warn!("{}", message);
}