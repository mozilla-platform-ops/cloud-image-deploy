use chrono::{
    DateTime,
    Utc
};
use serde::{
    Serialize,
    Deserialize
};
use std::env;
use taskcluster::{
    Auth,
    ClientBuilder,
    Credentials,
    //Index,
    //Queue,
    WorkerManager
};

pub struct TaskclusterClient {
    pub auth: Auth,
    /*
    pub index: Index,
    pub queue: Queue,
    */
    pub worker_manager: WorkerManager,
}
impl TaskclusterClient {
    pub fn new(root_url: &str) -> Self {
        TaskclusterClient {
            auth: match Auth::new(TaskclusterClient::get_client_builder(root_url)) {
                Ok(client) => client,
                Err(client_instantiation_error) => panic!("auth client instantiation error: {:?}", client_instantiation_error)
            },
            /*
            index: match Index::new(TaskclusterClient::get_client_builder(root_url)) {
                Ok(client) => client,
                Err(client_instantiation_error) => panic!("index client instantiation error: {:?}", client_instantiation_error)
            },
            queue: match Queue::new(TaskclusterClient::get_client_builder(root_url)) {
                Ok(client) => client,
                Err(client_instantiation_error) => panic!("queue client instantiation error: {:?}", client_instantiation_error)
            },
            */
            worker_manager: match WorkerManager::new(TaskclusterClient::get_client_builder(root_url)) {
                Ok(client) => client,
                Err(client_instantiation_error) => panic!("worker manager client instantiation error: {:?}", client_instantiation_error)
            }
        }
    }
    fn get_client_builder(root_url: &str) -> ClientBuilder {
        // use the proxy if TASKCLUSTER_PROXY_URL is set
        let url_override = match env::var("TASKCLUSTER_PROXY_URL") {
            Ok(x) => x,
            _ => root_url.to_string()
        };
        // a convenience for running locally, with credentials
        match Credentials::from_env() {
            // use credentials from env if set
            Ok(x) => ClientBuilder::new(&url_override).credentials(x),
            // fallback to anonymous or proxied client
            _ => ClientBuilder::new(&url_override)
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Client {
    client_id: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    created: Option<DateTime<Utc>>,

    #[serde(default = "default_false")]
    delete_on_expiration: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,

    #[serde(default = "default_false")]
    disabled: bool,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    expanded_scopes: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    expires: Option<DateTime<Utc>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    last_date_used: Option<DateTime<Utc>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    last_modified: Option<DateTime<Utc>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    last_rotated: Option<DateTime<Utc>>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    scopes: Vec<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerPool {
    config: WorkerPoolConfig,

    #[serde(skip_serializing_if = "Option::is_none")]
    created: Option<DateTime<Utc>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    current_capacity: Option<usize>,

    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,

    #[serde(default = "default_false")]
    email_on_error: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    last_modified: Option<DateTime<Utc>>,

    owner: String,

    provider_id: String,

    worker_pool_id: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerPoolConfig {
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    launch_configs: Vec<LaunchConfigContainer>,

    #[serde(skip_serializing_if = "Option::is_none")]
    lifecycle: Option<Lifecycle>,

    #[serde(skip_serializing_if = "Option::is_none")]
    max_capacity: Option<usize>,

    #[serde(skip_serializing_if = "Option::is_none")]
    min_capacity: Option<usize>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Lifecycle {
    #[serde(skip_serializing_if = "Option::is_none")]
    registration_timeout: Option<usize>,

    #[serde(skip_serializing_if = "Option::is_none")]
    reregistration_timeout: Option<usize>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenericWorkerConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    caches_dir: Option<String>,

    #[serde(default = "default_false")]
    clean_up_task_dirs: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    deployment_id: Option<String>,

    #[serde(default = "default_false")]
    disable_reboots: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    downloads_dir: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    ed25519_signing_key_location: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    idle_timeout_secs: Option<usize>,

    #[serde(skip_serializing_if = "Option::is_none")]
    livelog_executable: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "livelogPUTPort")]
    livelog_put_port: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    number_of_tasks_to_run: Option<usize>,

    #[serde(skip_serializing_if = "Option::is_none")]
    run_after_user_creation: Option<String>,

    #[serde(default = "default_false")]
    run_tasks_as_current_user: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    sentry_project: Option<String>,

    #[serde(default = "default_false")]
    shutdown_machine_on_idle: bool,

    #[serde(default = "default_false")]
    shutdown_machine_on_internal_error: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    taskcluster_proxy_executable: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    taskcluster_proxy_port: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    tasks_dir: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    wst_audience: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "wstServerURL")]
    wst_server_url: Option<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    capacity: Option<usize>,

    #[serde(skip_serializing_if = "Option::is_none")]
    generic_worker: Option<GenericWorker>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenericWorker {
    #[serde(skip_serializing_if = "Option::is_none")]
    config: Option<GenericWorkerConfig>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchConfigContainer {
    #[serde(skip_serializing_if = "Option::is_none")]
    additional_user_data: Option<AdditionalUserData>,

    #[serde(skip_serializing_if = "Option::is_none")]
    capacity_per_instance: Option<usize>,

    #[serde(skip_serializing_if = "Option::is_none")]
    launch_config: Option<LaunchConfig>,

    region: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    worker_config: Option<WorkerConfig>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchConfig {
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    #[serde(rename = "BlockDeviceMappings")]
    block_device_mappings: Vec<BlockDeviceMapping>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "ImageId")]
    image_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "InstanceMarketOptions")]
    instance_market_options: Option<InstanceMarketOptions>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "InstanceType")]
    instance_type: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "Placement")]
    placement: Option<Placement>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    #[serde(rename = "SecurityGroupIds")]
    security_group_ids: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "SubnetId")]
    subnet_id: Option<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdditionalUserData {
    #[serde(skip_serializing_if = "Option::is_none")]
    worker_type: Option<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceMarketOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "MarketType")]
    market_type: Option<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Placement {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "AvailabilityZone")]
    availability_zone: Option<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockDeviceMapping {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "DeviceName")]
    device_name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "Ebs")]
    ebs: Option<Ebs>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ebs {
    #[serde(default = "default_false")]
    #[serde(rename = "DeleteOnTermination")]
    delete_on_termination: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "VolumeSize")]
    volume_size: Option<usize>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "VolumeType")]
    volume_type: Option<String>,
}

fn default_false() -> bool {
    false
}