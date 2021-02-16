
use chrono::{DateTime, Utc};
use reqwest;
use serde::{
    Serialize,
    Deserialize
};
use std::collections::HashMap;
use std::env;
use taskcluster::{
    ClientBuilder,
    Credentials,
    Index,
    Queue,
    WorkerManager
};


struct Taskcluster {
    index: Index,
    queue: Queue,
    worker_manager: WorkerManager,
}
impl Taskcluster {
  pub fn new() -> Self {
    Taskcluster {
        index: match Index::new(get_client_builder()) {
            Ok(client) => client,
            Err(index_client_instantiation_error) => panic!("index_client_instantiation_error: {:?}", index_client_instantiation_error)
        },
        queue: match Queue::new(get_client_builder()) {
            Ok(client) => client,
            Err(queue_client_instantiation_error) => panic!("queue_client_instantiation_error: {:?}", queue_client_instantiation_error)
        },
        worker_manager: match WorkerManager::new(get_client_builder()) {
            Ok(client) => client,
            Err(worker_manager_client_instantiation_error) => panic!("worker_manager_client_instantiation_error: {:?}", worker_manager_client_instantiation_error)
        }
    }
  }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WorkerPool {
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

    #[serde(skip_serializing_if = "Option::is_none")]
    worker_pool_id: Option<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WorkerPoolConfig {
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
struct Lifecycle {
    #[serde(skip_serializing_if = "Option::is_none")]
    registration_timeout: Option<usize>,

    #[serde(skip_serializing_if = "Option::is_none")]
    reregistration_timeout: Option<usize>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GenericWorkerConfig {
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
struct WorkerConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    capacity: Option<usize>,

    #[serde(skip_serializing_if = "Option::is_none")]
    generic_worker: Option<GenericWorker>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GenericWorker {
    #[serde(skip_serializing_if = "Option::is_none")]
    config: Option<GenericWorkerConfig>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LaunchConfigContainer {
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
struct LaunchConfig {
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
struct AdditionalUserData {
    #[serde(skip_serializing_if = "Option::is_none")]
    worker_type: Option<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct InstanceMarketOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "MarketType")]
    market_type: Option<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Placement {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "AvailabilityZone")]
    availability_zone: Option<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BlockDeviceMapping {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "DeviceName")]
    device_name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "Ebs")]
    ebs: Option<Ebs>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Ebs {
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

#[tokio::main]
async fn main() {
    let tc = Taskcluster::new();
    let mut continuation_token: Option<String> = None;
    // list active pools
    loop {
        match tc.worker_manager.listWorkerPools(continuation_token.as_deref(), None).await {
            Ok(worker_pools_container) => {
                for worker_pool_container in worker_pools_container.get("workerPools").unwrap().as_array().unwrap() {
                    match [worker_pool_container.get("providerId").unwrap().as_str(), worker_pool_container.get("workerPoolId").unwrap().as_str(), worker_pool_container.get("owner").unwrap().as_str()] {
                        [Some(provider), Some(pool), Some(owner)] if (pool.contains("gecko") || pool.contains("relops")) && pool.contains("win") => println!(" - {}/{} ({})", provider, pool, owner),
                        _ => {}
                    }
                }
                match worker_pools_container.get("continuationToken") {
                    Some(v) => {
                        continuation_token = Some(v.as_str().unwrap().to_owned());
                    },
                    _ => {
                        continuation_token = None;
                        break;
                    }
                };
            },
            Err(list_clients_error) => panic!("list_clients_error: {:?}", list_clients_error)
        };
    };
    loop {
        match tc.index.listTasks("project.releng.opencloudconfig.v1.revision.latest", continuation_token.as_deref(), None).await {
            Ok(tasks_container) => {
                for task_container in tasks_container.get("tasks").unwrap().as_array().unwrap() {
                    match [task_container.get("taskId").unwrap().as_str(), task_container.get("namespace").unwrap().as_str()] {
                        [Some(task_id), Some(namespace)] if namespace.ends_with("-beta") => {
                            match tc.queue.getLatestArtifact_url(task_id, "public/ami-latest.json") {
                                Ok(artifact_url) => {
                                    let worker_type = match namespace.rsplit('.').next() {
                                        Some(x) => x,
                                        None => panic!("worker_type_determination_error: {}", namespace)
                                    };
                                    let template_pool_name = match worker_type {
                                        "gecko-1-b-win2012-beta" => "gecko-1/b-win2012",
                                        "gecko-t-win10-64-beta" => "gecko-t/t-win10-64",
                                        "gecko-t-win7-32-beta" => "gecko-t/t-win10-64",
                                        _ => panic!("pool_template_determination_error: {}", worker_type)
                                    };
                                    println!(" - {} (pool template: {})", worker_type, template_pool_name);
                                    match reqwest::get(&artifact_url).await {
                                        Ok(response) => match response.json::<HashMap<String, String>>().await {
                                            Ok(amis) => {
                                                match tc.worker_manager.workerPool(template_pool_name).await {
                                                    Ok(template_pool) => {
                                                        for launch_config_container in template_pool.get("config").unwrap().as_object().unwrap().get("launchConfigs").unwrap().as_array().unwrap() {
                                                            match [
                                                                launch_config_container.get("region").unwrap().as_str(),
                                                                launch_config_container.get("launchConfig").unwrap().as_object().unwrap().get("Placement").unwrap().as_object().unwrap().get("AvailabilityZone").unwrap().as_str(),
                                                                launch_config_container.get("launchConfig").unwrap().as_object().unwrap().get("ImageId").unwrap().as_str()
                                                            ] {
                                                                [Some(region), Some(az), Some(template_ami_id)] => {
                                                                    println!("   - {}", az);
                                                                    println!("     - old: {}", template_ami_id);
                                                                    let new_ami_id = match amis.get(region) {
                                                                        Some(x) => x.as_str(),
                                                                        _ => ""
                                                                    };
                                                                    println!("     - new: {}", new_ami_id);
                                                                },
                                                                _ => {}

                                                            }
                                                        }
                                                        let worker_pool: WorkerPool = match serde_json::from_value(template_pool) {
                                                            Ok(worker_pool) => worker_pool,
                                                            Err(worker_pool_deserialize_error) => panic!("worker_pool_deserialize_error: {:?}", worker_pool_deserialize_error)
                                                        };
                                                        println!("{:#?}", worker_pool);
                                                    },
                                                    Err(get_template_pool_error) => println!("get_template_pool_error: {:?}", get_template_pool_error)
                                                };
                                                //println!("{:#?}", amis);
                                            },
                                            Err(ami_latest_parse_error) => println!("ami_latest_parse_error ({}): {:?}", &artifact_url, ami_latest_parse_error)
                                        },
                                        Err(artifact_fetch_error) => println!("artifact_fetch_error ({}): {:?}", &artifact_url, artifact_fetch_error)
                                    };
                                },
                                Err(get_latest_artifact_url_error) => println!("get_latest_artifact_url_error: {:?}", get_latest_artifact_url_error)
                            };
                        },
                        _ => {}
                    };
                }
                match tasks_container.get("continuationToken") {
                    Some(v) => {
                        continuation_token = Some(v.as_str().unwrap().to_owned());
                    },
                    _ => break
                };
            },
            Err(list_tasks_error) => panic!("list_tasks_error: {:?}", list_tasks_error)
        };
    };
    /*
    // list occ build shas
    loop {
        match index_client.listNamespaces("project.releng.opencloudconfig.v1.revision", continuation_token.as_deref(), None).await {
            Ok(namespaces_container) => {
                for namespace_container in namespaces_container.get("namespaces").unwrap().as_array().unwrap() {
                    match [namespace_container.get("expires").unwrap().as_str(), namespace_container.get("name").unwrap().as_str(), namespace_container.get("namespace").unwrap().as_str()] {
                        [Some(expires), Some(name), Some(namespace)] => println!(" - {}/{} ({})", expires, name, namespace),
                        _ => {}
                    }
                }
                match namespaces_container.get("continuationToken") {
                    Some(v) => {
                        continuation_token = Some(v.as_str().unwrap().to_owned());
                    },
                    _ => break
                };
            },
            Err(list_namespaces_error) => panic!("list_namespaces_error: {:?}", list_namespaces_error)
        };
    }
    */
}

fn get_client_builder() -> ClientBuilder {
    // a convenience for running locally, instead of in a task where the proxy feature is enabled
    let root_url = match env::var("TASKCLUSTER_PROXY_URL") {
        Ok(x) => x,
        _ => match env::var("TASKCLUSTER_ROOT_URL") {
            Ok(x) => x,
            // fallback to staging if TASKCLUSTER_ROOT_URL is not set
            _ => "https://stage.taskcluster.nonprod.cloudops.mozgcp.net".to_string()
        }
    };
    // a convenience for running locally, with credentials
    match Credentials::from_env() {
        // use credentials from env if set
        Ok(x) => ClientBuilder::new(&root_url).credentials(x),
        // fallback to anonymous client
        _ => ClientBuilder::new(&root_url)
    }
}


fn default_false() -> bool {
    false
}