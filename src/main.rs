pub mod tc;

use chrono::{
    Duration,
    Utc
};
use clap::{
    App,
    load_yaml
};
use glob::glob;
use phf::{
    phf_map,
    phf_set
};
use itertools::Itertools;
use regex::Regex;
use reqwest::header;
use serde_json::json;
use slugid::nice;
use std::{
    collections::HashMap,
    env,
    fs::File,
};


const TASKCLUSTER_DEPLOYMENTS: phf::Map<&'static str, &'static str> = phf_map! {
    "https://community-tc.services.mozilla.com" => "community",
    "https://firefox-ci-tc.services.mozilla.com" => "production",
    "https://stage.taskcluster.nonprod.cloudops.mozgcp.net" => "staging",
};
const TASKCLUSTER_ENTITIES: phf::Set<&'static str> = phf_set! {
    "client",
    "role",
    "workerPool",
};
const WORKER_POOL_IMAGE_BUILDER: phf::Map<&'static str, &'static str> = phf_map! {
    "gecko-1/b-win2012" => "github.com/mozilla-releng/OpenCloudConfig",
    "gecko-1/b-win2012-beta" => "github.com/mozilla-releng/OpenCloudConfig",
    "gecko-3/b-win2012" => "github.com/mozilla-releng/OpenCloudConfig",
    "gecko-t/t-win10-64" => "github.com/mozilla-releng/OpenCloudConfig",
    "gecko-t/t-win10-64-gpu" => "github.com/mozilla-releng/OpenCloudConfig",
    "gecko-t/t-win10-64-gpu-s" => "github.com/mozilla-releng/OpenCloudConfig",
    "gecko-t/t-win7-32" => "github.com/mozilla-releng/OpenCloudConfig",
    "gecko-t/t-win7-32-gpu" => "github.com/mozilla-releng/OpenCloudConfig",
};

struct WorkerPoolMutation {
    pub machine_image: HashMap<String, String>,
    pub deployment_id: String,
}

#[tokio::main]
async fn main() {
    let args_config = load_yaml!("../.cli-arg-config.yml");
    let args = App::from_yaml(args_config).get_matches();
    let taskcluster_root_url = args.value_of("taskcluster-root-url").unwrap().trim_end_matches('/');
    let taskcluster_proxy_url = match env::var("TASKCLUSTER_PROXY_URL") { Ok(x) => x, _ => "".to_string() };

    println!("taskcluster-deployment-name: {}", TASKCLUSTER_DEPLOYMENTS[taskcluster_root_url]);
    println!("taskcluster-root-url: {}", taskcluster_root_url);
    println!("taskcluster-proxy-url: {}", taskcluster_proxy_url);
    for entity in TASKCLUSTER_ENTITIES.into_iter() {
        println!("{}-pattern: {}", entity, args.value_of(format!("{}-pattern", entity)).unwrap());
    }

    if let Some(subcommand_args) = args.subcommand_matches("deploy") {
        for entity in TASKCLUSTER_ENTITIES.into_iter() {
            deploy(entity, subcommand_args).await;
        }
    }

    if let Some(subcommand_args) = args.subcommand_matches("snapshot") {
        for entity in TASKCLUSTER_ENTITIES.into_iter() {
            snapshot(entity, subcommand_args).await;
        }
    }

    if let Some(subcommand_args) = args.subcommand_matches("mutate") {
        for entity in TASKCLUSTER_ENTITIES.into_iter() {
            mutate(entity, subcommand_args).await;
        }
    }
}

async fn deploy(entity: &str, args: &clap::ArgMatches<'_>) {
    let taskcluster_root_url = args.value_of("taskcluster-root-url").unwrap().trim_end_matches('/');
    let taskcluster_client = tc::TaskclusterClient::new(taskcluster_root_url);
    let deploy_folder = format!("{}/{}/{}", args.value_of("deploy-folder").unwrap().trim_end_matches('/'), TASKCLUSTER_DEPLOYMENTS[taskcluster_root_url], entity);
    println!("deploy-folder: {}", deploy_folder);
    match glob(&format!("{}/**/*.yml", deploy_folder)) {
        Ok(entries) => {
          for entry in entries {
            match entry {
                Ok(path_buf) => match path_buf.to_str() {
                    Some(item_yaml_file_path) => {
                        let item_id = &item_yaml_file_path[(deploy_folder.len() + 1)..(item_yaml_file_path.len() - 4)];
                        if Regex::new(args.value_of(format!("{}-pattern", entity)).unwrap()).unwrap().is_match(item_id) {
                            println!("processing {} deploy: {}...", entity, item_id);
                            let item_yaml_value: serde_yaml::Value = match std::fs::File::open(item_yaml_file_path) {
                                Ok(item_yaml_file) => match serde_yaml::from_reader(item_yaml_file) {
                                    Ok(item_yaml_value) => item_yaml_value,
                                    Err(item_yaml_parse_error) => panic!("parse {} yaml error: {:?}", entity, item_yaml_parse_error)
                                },
                                Err(item_yaml_open_error) => panic!("open {} yaml error: {:?}", entity, item_yaml_open_error)
                            };
                            let local_item_json_value: serde_json::Value = serde_json::to_value(item_yaml_value).unwrap();
                            let item_create_or_update_response = match entity {
                                "workerPool" => match taskcluster_client.worker_manager.workerPool(item_id).await {
                                    Ok(_remote_item_json_value) => {
                                        // item exists on remote, use update
                                        // todo: skip update if local does not differ from remote
                                        match taskcluster_client.worker_manager.updateWorkerPool(item_id, &local_item_json_value).await {
                                            Ok(item_update_response) => item_update_response,
                                            Err(item_update_error) => panic!("{} update error: {:?}", entity, item_update_error)
                                        };
                                    },
                                    _ => {
                                        // item does not exist on remote, use create
                                        match taskcluster_client.worker_manager.createWorkerPool(item_id, &local_item_json_value).await {
                                            Ok(item_create_response) => item_create_response,
                                            Err(item_create_error) => panic!("{} create error: {:?}", entity, item_create_error)
                                        };
                                    },
                                },
                                unsupported_entity => panic!("unsupported entity: {}", unsupported_entity)
                            };
                            serde_json::to_writer(std::io::stdout(), &item_create_or_update_response).unwrap();
                            match entity {
                                "workerPool" => match item_id.splitn(2, '/').collect_tuple() {
                                    Some((domain, pool)) => {
                                        let task_definition = json!({
                                            "created": Utc::now(),
                                            "deadline": Utc::now() + Duration::hours(1),
                                            "dependencies": [
                                                std::env::var("TASK_ID").unwrap()
                                            ],
                                            "provisionerId": domain,
                                            "workerType": pool,
                                            "priority": "highest",
                                            "payload": {
                                                "command": [
                                                    &format!("echo \"heartbeat from {}\"", item_id),
                                                ],
                                                "env": {
                                                    "GITHUB_HEAD_SHA": std::env::var("GITHUB_HEAD_SHA").unwrap(),
                                                },
                                                "maxRunTime": 60,
                                            },
                                            "metadata": {
                                                "name": &format!("02 :: validate {}", item_id),
                                                "description": &format!("validate task worthiness of {} after pool config mutation", item_id),
                                                "owner": "relops@mozilla.com",
                                                "source": &format!("https://github.com/mozilla-platform-ops/cloud-image-deploy/commit/{}", std::env::var("GITHUB_HEAD_SHA").unwrap()),
                                            },
                                            "schedulerId": "taskcluster-github",
                                            "taskGroupId": std::env::var("TASK_GROUP_ID").unwrap()
                                        });
                                        match taskcluster_client.queue.createTask(&nice(), &task_definition).await {
                                            Ok(create_task_result) => println!("create_task_result: {:?}", create_task_result),
                                            Err(create_task_error) => panic!("create_task_error: {:?}", create_task_error),
                                        };
                                    },
                                    None => panic!("failed to parse domain/pool from task queue id")
                                },
                                unsupported_entity => panic!("unsupported entity: {}", unsupported_entity)
                            };
                        } else {
                            println!("skipping {} deploy: {}...", entity, item_id);
                        }
                    },
                    None => {}
                },
                Err(path_read_error) => println!("path_read_error: {:?}", path_read_error)
            }
          }
        },
        Err(glob_pattern_read_error) => println!("glob_pattern_read_error: {:?}", glob_pattern_read_error)
    };
}

async fn snapshot(entity: &str, args: &clap::ArgMatches<'_>) {
    let taskcluster_root_url = args.value_of("taskcluster-root-url").unwrap().trim_end_matches('/');
    let taskcluster_client = tc::TaskclusterClient::new(taskcluster_root_url);
    let snapshot_folder = args.value_of("snapshot-folder").unwrap().trim_end_matches('/');
    println!("snapshot-folder: {}/{}/{}", snapshot_folder, TASKCLUSTER_DEPLOYMENTS[taskcluster_root_url], entity);

    let mut continuation_token: Option<String> = None;
    loop {
        let list_json = match entity {
            "client" => match taskcluster_client.auth.listClients(None, continuation_token.as_deref(), None).await {
                Ok(list) => list,
                Err(list_items_error) => panic!("list {}s error: {:?}", entity, list_items_error)
            },
            "role" => match taskcluster_client.auth.listRoles2(continuation_token.as_deref(), None).await {
                Ok(list) => list,
                Err(list_items_error) => panic!("list {}s error: {:?}", entity, list_items_error)
            },
            "workerPool" => match taskcluster_client.worker_manager.listWorkerPools(continuation_token.as_deref(), None).await {
                Ok(list) => list,
                Err(list_items_error) => panic!("list {}s error: {:?}", entity, list_items_error)
            },
            unsupported_entity => panic!("unsupported entity: {}", unsupported_entity)
        };
        for item_json_value in list_json.get(format!("{}s", entity)).unwrap().as_array().unwrap().iter() {
            let item_id = item_json_value.get(format!("{}Id", entity)).unwrap().as_str().unwrap();
            if Regex::new(args.value_of(format!("{}-pattern", entity)).unwrap()).unwrap().is_match(item_id) {
                println!("processing snapshot {} config: {}...", entity, item_id);
                let item_yaml_value: serde_yaml::Value = serde_yaml::to_value(item_json_value).unwrap();
                serde_yaml::to_writer(std::io::stdout(), &item_yaml_value).unwrap();
                let item_yaml_file_path = format!("{}/{}/{}/{}.yml", snapshot_folder, TASKCLUSTER_DEPLOYMENTS[taskcluster_root_url], entity, item_id);
                std::fs::create_dir_all(std::path::Path::new(&item_yaml_file_path).parent().unwrap()).unwrap();
                match File::create(item_yaml_file_path) {
                    Ok(item_yaml_file) => serde_yaml::to_writer(item_yaml_file, &item_yaml_value).unwrap(),
                    Err(create_item_yaml_file_error) => println!("create {} yaml file error: {:?}", entity, create_item_yaml_file_error)
                };
            } else {
                println!("skipping {} snapshot: {}...", entity, item_id);
            }
        }
        match list_json.get("continuationToken") {
            Some(v) => {
                continuation_token = Some(v.as_str().unwrap().to_owned());
            },
            _ => break
        };
    };
}

async fn mutate(entity: &str, args: &clap::ArgMatches<'_>) {
    let taskcluster_root_url = args.value_of("taskcluster-root-url").unwrap().trim_end_matches('/');
    let taskcluster_client = tc::TaskclusterClient::new(taskcluster_root_url);
    let deploy_folder = format!("{}/{}/{}", args.value_of("deploy-folder").unwrap().trim_end_matches('/'), TASKCLUSTER_DEPLOYMENTS[taskcluster_root_url], entity);
    println!("deploy-folder: {}", deploy_folder);
    let snapshot_folder = format!("{}/{}/{}", args.value_of("snapshot-folder").unwrap().trim_end_matches('/'), TASKCLUSTER_DEPLOYMENTS[taskcluster_root_url], entity);
    println!("snapshot-folder: {}", snapshot_folder);
    match glob(&format!("{}/**/*.yml", snapshot_folder)) {
        Ok(entries) => {
            for entry in entries {
                match entry {
                    Ok(path_buf) => match path_buf.to_str() {
                        Some(item_snapshot_yaml_file_path) => {
                            let item_id = &item_snapshot_yaml_file_path[(snapshot_folder.len() + 1)..(item_snapshot_yaml_file_path.len() - 4)];
                            if Regex::new(args.value_of(format!("{}-pattern", entity)).unwrap()).unwrap().is_match(item_id) {
                                println!("processing mutate {} config: {}...", entity, item_id);
                                let mut item_yaml_value: serde_yaml::Value = match std::fs::File::open(item_snapshot_yaml_file_path) {
                                    Ok(item_snapshot_yaml_file) => match serde_yaml::from_reader(item_snapshot_yaml_file) {
                                        Ok(item_yaml_value) => item_yaml_value,
                                        Err(item_yaml_parse_error) => panic!("parse {} yaml error: {:?}", entity, item_yaml_parse_error)
                                    },
                                    Err(item_yaml_open_error) => panic!("open {} yaml error: {:?}", entity, item_yaml_open_error)
                                };
                                match entity {
                                    "workerPool" => {
                                        let reqwest_client = reqwest::Client::new();
                                        let mutation: WorkerPoolMutation = match args.value_of("build-sha") {
                                            Some(arg_build_sha) if arg_build_sha == "latest" => {
                                                if WORKER_POOL_IMAGE_BUILDER.contains_key(item_id) {
                                                    panic!("no implementation to lookup build-sha latest for {} from {}", item_id, WORKER_POOL_IMAGE_BUILDER[item_id]);
                                                    //todo: use taskcluster_client.index to lookup the latest build sha
                                                } else {
                                                    panic!("no implementation to determine build-sha latest for {}", item_id);
                                                }
                                            },
                                            Some(arg_build_sha) if Regex::new("[A-Za-z0-9_.-]*/[A-Za-z0-9_.-]*/[0-9a-fA-F]{7,40}").unwrap().is_match(arg_build_sha) => match arg_build_sha.splitn(3, '/').collect_tuple() {
                                                Some((owner, repo, sha)) => match reqwest_client.get(&format!("https://api.github.com/repos/{}/{}/commits/{}/statuses", owner, repo, sha)).header(header::USER_AGENT, "mozilla-platform-ops/cloud-image-deploy").send().await {
                                                    Ok(github_statuses_response) => match github_statuses_response.json::<serde_json::Value>().await {
                                                        Ok(github_statuses) => match github_statuses.as_array().unwrap().iter().find(|status| status.get("state").unwrap().as_str().unwrap() == "success") {
                                                            Some(github_status) => match github_status.get("target_url") {
                                                                Some(task_group_url_node) => match task_group_url_node.as_str() {
                                                                    Some(task_group_url) => match task_group_url.split('/').next_back() {
                                                                        Some(task_group_id) => match taskcluster_client.queue.listTaskGroup(task_group_id, None, None).await {
                                                                            Ok(task_group) => match task_group.get("tasks") {
                                                                                Some(tasks_node) => match tasks_node.as_array() {
                                                                                    Some(tasks) => match tasks.iter().find(
                                                                                        |x| {
                                                                                            // filter for completed task with worker type in name
                                                                                            x.as_object().unwrap().get("task").unwrap().as_object().unwrap().get("metadata").unwrap().as_object().unwrap().get("name").unwrap().as_str().unwrap() == &format!("Update {} AMIs", item_id.replace("/", "-").replace("gecko-t-t-", "gecko-t-"))
                                                                                            && x.as_object().unwrap().get("status").unwrap().as_object().unwrap().get("state").unwrap().as_str().unwrap() == "completed"
                                                                                        }) {
                                                                                        Some(task_container) => match taskcluster_client.queue.getLatestArtifact_url(task_container.as_object().unwrap().get("status").unwrap().as_object().unwrap().get("taskId").unwrap().as_str().unwrap(), "public/ami-latest.json") {
                                                                                            Ok(ami_latest_artifact_url) => match reqwest_client.get(&ami_latest_artifact_url).send().await {
                                                                                                Ok(ami_latest_artifact_response) => match ami_latest_artifact_response.json::<std::collections::HashMap<String, String>>().await {
                                                                                                    Ok(ami_latest) => WorkerPoolMutation {
                                                                                                        machine_image: ami_latest,
                                                                                                        deployment_id: sha[..7].to_string(),
                                                                                                    },
                                                                                                    Err(parse_ami_latest_artifact_error) => panic!("parse_ami_latest_artifact_error: {:?}, url: {}", parse_ami_latest_artifact_error, ami_latest_artifact_url)
                                                                                                },
                                                                                                Err(fetch_ami_latest_artifact_error) => panic!("fetch_ami_latest_artifact_error: {:?}, url: {}", fetch_ami_latest_artifact_error, ami_latest_artifact_url)
                                                                                            },
                                                                                            Err(ami_latest_artifact_url_error) => panic!("failed to determine url for artifact: public/ami-latest.json in task: {}. {:?}", task_container.as_object().unwrap().get("status").unwrap().as_object().unwrap().get("taskId").unwrap().as_str().unwrap(), ami_latest_artifact_url_error)
                                                                                        },
                                                                                        None => panic!("task group {} does not contain an image build task for {}", task_group_id, item_id)
                                                                                    },
                                                                                    None => panic!("tasks parse failure")
                                                                                },
                                                                                None => panic!("tasks missing in task group")
                                                                            },
                                                                            Err(list_task_group_error) => panic!("list task group {} error: {:?}", task_group_id, list_task_group_error)
                                                                        },
                                                                        None => panic!("task_group_id parse failure")
                                                                    },
                                                                    None => panic!("task_group_url parse failure")
                                                                },
                                                                None => panic!("target_url missing in github status")
                                                            },
                                                            None => panic!("build-sha {}/{}/{} does not reference a successfully completed image build", owner, repo, sha)
                                                        },
                                                        Err(parse_github_statuses_error) => panic!("parse_github_statuses_error: {:?}, url: {}", parse_github_statuses_error, format!("https://api.github.com/repos/{}/{}/commits/{}/statuses", owner, repo, sha))
                                                    },
                                                    Err(fetch_github_statuses_error) => panic!("fetch_github_statuses_error: {:?}, url: {}", fetch_github_statuses_error, format!("https://api.github.com/repos/{}/{}/commits/{}/statuses", owner, repo, sha))
                                                },
                                                None => panic!("failed to determine owner, repo, sha using supplied build-sha")
                                            },
                                            Some(arg_build_sha) => panic!("no implementation to handle build-sha {}", arg_build_sha),
                                            None => panic!("missing required argument: build-sha")
                                        };
                                        match item_yaml_value["config"].as_mapping_mut() {
                                            Some(config) => match config[&serde_yaml::Value::String("launchConfigs".into())].as_sequence_mut() {
                                                Some(launch_configs) => {
                                                    for zone_config_value in launch_configs {
                                                        match zone_config_value.as_mapping_mut() {
                                                            Some(zone_config) => {
                                                                match zone_config[&serde_yaml::Value::String("region".into())].as_str() {
                                                                    Some(region) => {
                                                                        zone_config[&serde_yaml::Value::String("launchConfig".into())]["ImageId"] = serde_yaml::to_value(&mutation.machine_image[region]).unwrap();
                                                                        zone_config[&serde_yaml::Value::String("workerConfig".into())]["genericWorker"]["config"]["deploymentId"] = serde_yaml::to_value(&mutation.deployment_id).unwrap();
                                                                        // older gw versions expected workerConfig.capacity & workerConfig.genericWorker.config.livelogPUTPort. newer gw versions will error out and die if the fields exist.
                                                                        match zone_config[&serde_yaml::Value::String("workerConfig".into())]
                                                                            .as_mapping_mut().unwrap().remove(&serde_yaml::Value::String("capacity".to_string())) { _ => {} }
                                                                        match zone_config[&serde_yaml::Value::String("workerConfig".into())]
                                                                            .as_mapping_mut().unwrap()[&serde_yaml::Value::String("genericWorker".into())]
                                                                            .as_mapping_mut().unwrap()[&serde_yaml::Value::String("config".into())]
                                                                            .as_mapping_mut().unwrap().remove(&serde_yaml::Value::String("livelogPUTPort".to_string())) { _ => {} }
                                                                    },
                                                                    None => {}
                                                                }
                                                            },
                                                            None => {}
                                                        }
                                                    }
                                                },
                                                None => {}
                                            },
                                            None => {}
                                        }
                                    },
                                    unsupported_entity => panic!("unsupported entity: {}", unsupported_entity)
                                };
                                match item_yaml_value.as_mapping_mut() {
                                    Some(item_yaml_mapping_mut) => {
                                        for disallowed_value in ["created", "currentCapacity", "lastModified", "workerPoolId"].iter() {
                                            match item_yaml_mapping_mut.remove(&serde_yaml::Value::String(disallowed_value.to_string())) { _ => {} }
                                        }
                                    },
                                    None => {}
                                };
                                item_yaml_value["description"] = serde_yaml::Value::String(format!("## {}\n### {}\n\nthis pool configuration is managed by [cloud-image-deploy](https://github.com/mozilla-platform-ops/cloud-image-deploy).\n\nto make persistent changes to this pool configuration, submit a pull request for [{}.yml](https://github.com/mozilla-platform-ops/cloud-image-deploy/blob/main/.deploy/production/workerPool/{}.yml).", item_id, TASKCLUSTER_DEPLOYMENTS[taskcluster_root_url], item_id, item_id));
                                if args.is_present("owner") {
                                    item_yaml_value["owner"] = serde_yaml::Value::String(args.value_of("owner").unwrap().to_string());
                                    item_yaml_value["emailOnError"] = serde_yaml::Value::Bool(true);
                                }
                                //serde_yaml::to_writer(std::io::stdout(), &item_yaml_value).unwrap();
                                let item_deploy_yaml_file_path = format!("{}/{}.yml", deploy_folder, item_id);
                                std::fs::create_dir_all(std::path::Path::new(&item_deploy_yaml_file_path).parent().unwrap()).unwrap();
                                match File::create(&item_deploy_yaml_file_path) {
                                    Ok(item_deploy_yaml_file) => match serde_yaml::to_writer(item_deploy_yaml_file, &item_yaml_value) {
                                        Ok(_) => println!("{} deploy yaml file written to: {}", entity, item_deploy_yaml_file_path),
                                        Err(write_item_deploy_yaml_file_error) => println!("write {} deploy yaml file error: {:?}", entity, write_item_deploy_yaml_file_error)
                                    },
                                    Err(create_item_deploy_yaml_file_error) => println!("create {} deploy yaml file error: {:?}", entity, create_item_deploy_yaml_file_error)
                                };
                            } else {
                                println!("skipping {} config update: {}...", entity, item_id);
                            }
                        },
                        None => {}
                    },
                    Err(path_read_error) => println!("path_read_error: {:?}", path_read_error)
                }
            }
        },
        Err(glob_pattern_read_error) => println!("glob_pattern_read_error: {:?}", glob_pattern_read_error)
    };
}
