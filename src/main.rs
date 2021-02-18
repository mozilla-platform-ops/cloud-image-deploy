pub mod tc;

use clap::{
    App,
    load_yaml
};
use glob::glob;
use phf::{
    phf_map,
    phf_set
};
use regex::Regex;
use std::{
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
                            println!("processing create or update {}: {}...", entity, item_id);
                            let item_yaml_value: serde_yaml::Value = match std::fs::File::open(item_yaml_file_path) {
                                Ok(item_yaml_file) => match serde_yaml::from_reader(item_yaml_file) {
                                    Ok(item_yaml_value) => item_yaml_value,
                                    Err(item_yaml_parse_error) => panic!("parse {} yaml error: {:?}", entity, item_yaml_parse_error)
                                },
                                Err(item_yaml_open_error) => panic!("open {} yaml error: {:?}", entity, item_yaml_open_error)
                            };
                            let local_item_json_value: serde_json::Value = serde_json::to_value(item_yaml_value).unwrap();
                            match entity {
                                "workerPool" => match taskcluster_client.worker_manager.workerPool(item_id).await {
                                    Ok(remote_item_json_value) => {
                                        // item exists on remote, use update
                                        // todo: skip update if local does not differ from remote
                                        match taskcluster_client.worker_manager.updateWorkerPool(item_id, &local_item_json_value).await {
                                            Ok(item_update_response) => serde_json::to_writer(std::io::stdout(), &item_update_response).unwrap(),
                                            Err(item_update_error) => panic!("{} update error: {:?}", entity, item_update_error)
                                        };
                                    },
                                    _ => {
                                        // item does not exist on remote, use create
                                        match taskcluster_client.worker_manager.createWorkerPool(item_id, &local_item_json_value).await {
                                            Ok(item_create_response) => serde_json::to_writer(std::io::stdout(), &item_create_response).unwrap(),
                                            Err(item_create_error) => panic!("{} create error: {:?}", entity, item_create_error)
                                        };
                                    },
                                },
                                unsupported_entity => panic!("unsupported entity: {}", unsupported_entity)
                            };
                        } else {
                            println!("skipping create or update {}: {}...", entity, item_id);
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
        for item_json_value in list_json.get(format!("{}s", entity)).unwrap().as_array().unwrap().iter().filter(|i| Regex::new(args.value_of(format!("{}-pattern", entity)).unwrap()).unwrap().is_match(i.get(format!("{}Id", entity)).unwrap().as_str().unwrap())) {
            let item_yaml_value: serde_yaml::Value = serde_yaml::to_value(item_json_value).unwrap();
            serde_yaml::to_writer(std::io::stdout(), &item_yaml_value).unwrap();
            let item_yaml_file_path = format!("{}/{}/{}/{}.yml", snapshot_folder, TASKCLUSTER_DEPLOYMENTS[taskcluster_root_url], entity, item_json_value.get(format!("{}Id", entity)).unwrap().as_str().unwrap());
            std::fs::create_dir_all(std::path::Path::new(&item_yaml_file_path).parent().unwrap()).unwrap();
            match File::create(item_yaml_file_path) {
                Ok(item_yaml_file) => serde_yaml::to_writer(item_yaml_file, &item_yaml_value).unwrap(),
                Err(create_item_yaml_file_error) => println!("create {} yaml file error: {:?}", entity, create_item_yaml_file_error)
            };
        }
        match list_json.get("continuationToken") {
            Some(v) => {
                continuation_token = Some(v.as_str().unwrap().to_owned());
            },
            _ => break
        };
    };
}
