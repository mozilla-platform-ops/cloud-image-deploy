use std::env;
use taskcluster::{
    Auth,
    ClientBuilder,
    Credentials,
    Index,
    Queue,
    WorkerManager
};

pub struct TaskclusterClient {
    pub auth: Auth,
    pub index: Index,
    pub queue: Queue,
    pub worker_manager: WorkerManager,
}
impl TaskclusterClient {
    pub fn new(root_url: &str) -> Self {
        TaskclusterClient {
            auth: match Auth::new(TaskclusterClient::get_client_builder(root_url)) {
                Ok(client) => client,
                Err(client_instantiation_error) => panic!("auth client instantiation error: {:?}", client_instantiation_error)
            },
            index: match Index::new(TaskclusterClient::get_client_builder(root_url)) {
                Ok(client) => client,
                Err(client_instantiation_error) => panic!("index client instantiation error: {:?}", client_instantiation_error)
            },
            queue: match Queue::new(TaskclusterClient::get_client_builder(root_url)) {
                Ok(client) => client,
                Err(client_instantiation_error) => panic!("queue client instantiation error: {:?}", client_instantiation_error)
            },
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
