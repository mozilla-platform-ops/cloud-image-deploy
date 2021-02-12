use std::env;
use taskcluster::{WorkerManager, ClientBuilder, Credentials};


#[tokio::main]
async fn main() {
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
    let api_client = match Credentials::from_env() {
        // use credentials from env if set
        Ok(x) => ClientBuilder::new(&root_url).credentials(x),
        // fallback to anonymous client
        _ => ClientBuilder::new(&root_url)
    };
    /*
    match Auth::new(api_client) {
        Ok(auth_client) => {
            let prefix: Option<String> = None; // filter on client prefix: Some("my_client_prefix")
            let mut continuation_token: Option<String> = None;
            let limit: Option<String> = None; // results per page: Some("10")
            loop {
                match auth_client.listClients(prefix.as_deref(), continuation_token.as_deref(), limit.as_deref()).await {
                    Ok(container) => {
                        for client in container.get("clients").unwrap().as_array().unwrap() {
                            println!("{:?}", client);
                        }
                        match container.get("continuationToken") {
                            Some(v) => {
                                continuation_token = Some(v.as_str().unwrap().to_owned());
                            },
                            _ => break
                        };
                    },
                    Err(list_clients_error) => panic!("list_clients_error: {:?}", list_clients_error)
                };
            }
        },
        Err(new_auth_client_error) => panic!("new_auth_client_error: {:?}", new_auth_client_error)
    };
    */
    match WorkerManager::new(api_client) {
        Ok(wm_client) => {
            let mut continuation_token: Option<String> = None;
            loop {
                match wm_client.listWorkerPools(continuation_token.as_deref(), None).await {
                    Ok(container) => {
                        for worker_pool in container.get("workerPools").unwrap().as_array().unwrap() {
                            match [worker_pool.get("providerId").unwrap().as_str(), worker_pool.get("workerPoolId").unwrap().as_str(), worker_pool.get("owner").unwrap().as_str()] {
                                [Some(provider), Some(pool), Some(owner)] if (pool.contains("gecko") || pool.contains("relops")) && pool.contains("win") => println!(" - {}/{} ({})", provider, pool, owner),
                                _ => {}
                            }
                        }
                        match container.get("continuationToken") {
                            Some(v) => {
                                continuation_token = Some(v.as_str().unwrap().to_owned());
                            },
                            _ => break
                        };
                    },
                    Err(list_clients_error) => panic!("list_clients_error: {:?}", list_clients_error)
                };
            }
        },
        Err(new_auth_client_error) => panic!("new_auth_client_error: {:?}", new_auth_client_error)
    };
}
