pub mod tc;

use clap::{
    App,
    load_yaml
};
use regex::Regex;
use std::{
    env,
    fs::File,
};

/*
usage:

cargo run -- snapshot -r https://firefox-ci-tc.services.mozilla.com -w "^(relops.*|(gecko|mpd001)-[1-3t]/([bt]-)?win.*)$" -p aws -c "relops" -l "relops|mozilla-platform-ops|OpenCloudConfig|worker-pool:.*win.*|workerPool.*win.*"
cargo run -- snapshot -r https://stage.taskcluster.nonprod.cloudops.mozgcp.net -w "^(relops.*|(gecko|mpd001)-[1-3t]/([bt]-)?win.*)$" -p aws -c "relops" -l "relops|mozilla-platform-ops|OpenCloudConfig|worker-pool:.*win.*|workerPool.*win.*"
*/

#[tokio::main]
async fn main() {
    let clap_yaml = load_yaml!("../.clap.yml");
    let clap_matches = App::from_yaml(clap_yaml).get_matches();
    let taskcluster_root_url = clap_matches.value_of("taskcluster-root-url").unwrap().trim_end_matches('/');
    let taskcluster_deployment_name = clap_matches.value_of("taskcluster-deployment-name").unwrap_or(match taskcluster_root_url {
        root_url if root_url == "https://community-tc.services.mozilla.com" => "community",
        root_url if root_url == "https://firefox-ci-tc.services.mozilla.com" => "production",
        root_url if root_url == "https://stage.taskcluster.nonprod.cloudops.mozgcp.net" => "staging",
        root_url => panic!("unknown taskcluster root url: {}", root_url)
    });
    let taskcluster_proxy_url = match env::var("TASKCLUSTER_PROXY_URL") { Ok(x) => x, _ => "".to_string() };

    let config_folder = clap_matches.value_of("config-folder").unwrap().trim_end_matches('/');

    let worker_pool_pattern = clap_matches.value_of("workerPool-pattern").unwrap();
    let provider_pattern = clap_matches.value_of("provider-pattern").unwrap();
    let owner_pattern = clap_matches.value_of("owner-pattern").unwrap();

    println!("taskcluster-deployment-name: {}", taskcluster_deployment_name);
    println!("taskcluster-root-url: {}", taskcluster_root_url);
    println!("taskcluster-proxy-url: {}", taskcluster_proxy_url);

    println!("config-folder: {}", config_folder);

    println!("worker-pool-pattern: {}", worker_pool_pattern);
    println!("provider-pattern: {}", provider_pattern);
    println!("owner-pattern: {}", owner_pattern);

    if let Some(subcommand_matches) = clap_matches.subcommand_matches("snapshot") {
        if subcommand_matches.is_present("observe-only") {
            println!("snapshot in observe-only mode");
        } else {
            println!("snapshot in write mode");
        }
        for item_name in &["client", "role", "workerPool"] {
            snapshot(item_name, config_folder, taskcluster_deployment_name, subcommand_matches).await;
        }
    }
}

async fn snapshot(item_name: &str, config_folder: &str, taskcluster_deployment_name: &str, matches: &clap::ArgMatches<'_>) {
    let taskcluster_client = tc::TaskclusterClient::new(matches.value_of("taskcluster-root-url").unwrap().trim_end_matches('/'));
    let mut continuation_token: Option<String> = None;
    loop {
        let list_json = match item_name {
            "client" => match taskcluster_client.auth.listClients(None, continuation_token.as_deref(), None).await {
                Ok(list) => list,
                Err(list_items_error) => panic!("list {}s error: {:?}", item_name, list_items_error)
            },
            "role" => match taskcluster_client.auth.listRoles2(continuation_token.as_deref(), None).await {
                Ok(list) => list,
                Err(list_items_error) => panic!("list {}s error: {:?}", item_name, list_items_error)
            },
            "workerPool" => match taskcluster_client.worker_manager.listWorkerPools(continuation_token.as_deref(), None).await {
                Ok(list) => list,
                Err(list_items_error) => panic!("list {}s error: {:?}", item_name, list_items_error)
            },
            unsupported_entity => panic!("unsupported entity: {}", unsupported_entity)
        };
        for item_json in list_json.get(format!("{}s", item_name)).unwrap().as_array().unwrap().iter().filter(|i| Regex::new(matches.value_of(format!("{}-pattern", item_name)).unwrap()).unwrap().is_match(i.get(format!("{}Id", item_name)).unwrap().as_str().unwrap())) {
            let item_yaml: serde_yaml::Value = serde_yaml::to_value(item_json).unwrap();
            if matches.is_present("observe-only") {
                serde_yaml::to_writer(std::io::stdout(), &item_yaml).unwrap();
            } else {
                serde_yaml::to_writer(std::io::stdout(), &item_yaml).unwrap();
                let item_yaml_file_path = format!("{}/{}/{}/{}.yml", config_folder, taskcluster_deployment_name, item_name, item_json.get(format!("{}Id", item_name)).unwrap().as_str().unwrap());
                std::fs::create_dir_all(std::path::Path::new(&item_yaml_file_path).parent().unwrap()).unwrap();
                match File::create(item_yaml_file_path) {
                    Ok(item_yaml_file) => serde_yaml::to_writer(item_yaml_file, &item_yaml).unwrap(),
                    Err(create_item_yaml_file_error) => println!("create {} yaml file error: {:?}", item_name, create_item_yaml_file_error)
                };
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