## cloud image deploy

this is a minimal implementation of a worker pool creator / updater.

it is useful for quickly and simply deploying one or more, new or updated worker pool definitions to multiple environments.

example use cases include:
* test updated cloud machine images before adding their id to ci-configuration
* test generic worker configuration changes
* stagger the rollout of a change whose impact at scale is unknown by modifying individual (regional/zonal) launch configurations one at a time instead of updating an entire pool (all availability zones) at the same time.

it is important to note that changes made by this application to taskcluster worker pool configurations are not permanent and will be overwritten any time ci-admin is run and the configurations here do not match the configurations in ci-configuration.

this repository and application is evolving. there are currently two distinct implementations.
* the new rust implemntation has three subcommand implementations:
  * the **snapshot** subcommand queries taskcluster endpoints and creates yaml configuration files representing its observations in the **/.snapshot** folder
  * the **mutate** subcommand queries bootstrap repository endpoints (so far just [occ](https://github.com/mozilla-releng/OpenCloudConfig), but a ronin implementation is envisaged) and mutates yaml configuration files from the **/.snapshot** folder into modified configurations in the **/.deploy** folder
  * the **deploy** subcommand interacts with taskcluster endpoints to deploy configurations found in the **/.deploy** folder
* the original python implementation deploys taskcluster configurations defined in the **/config** folder

### new rust implementation

#### installation

* clone this repository
  ```
  git clone https://github.com/mozilla-platform-ops/cloud-image-deploy.git
  ```
* install (rust) dependencies
  ```
  sudo dnf dnf install -y cmake gcc openssl-devel
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
  ```

#### usage

the most useful use case for this implementation is to create or update worker pools when new machine images (amis) have been built (or we want to revert to old amis). to accomplish this, use the following steps (example commands will modify all windows **beta** worker pools in the taskcluster **staging** deployment):
1. create a snapshot of existing taskcluster configurations (if they don't exist, snapshot a similar configuration and modify its filename and contents afterwards):
   ```
   cargo run -- snapshot \
     #--taskcluster-root-url https://firefox-ci-tc.services.mozilla.com \
     --taskcluster-root-url https://stage.taskcluster.nonprod.cloudops.mozgcp.net \
     --worker-pool-pattern "^gecko-[1t]/[bt]-win(7-32|10-64|2012)-(beta|gpu-b)$"
   ```
2. mutate the snapshot configurations with regional amis from a specific bootstrap image build sha:
   ```
   cargo run -- mutate \
     #--taskcluster-root-url https://firefox-ci-tc.services.mozilla.com \
     --taskcluster-root-url https://stage.taskcluster.nonprod.cloudops.mozgcp.net \
     --worker-pool-pattern "^gecko-[1t]/[bt]-win(7-32|10-64|2012)-(beta|gpu-b)$" \
     --build-sha "mozilla-releng/OpenCloudConfig/20d46de4453f42261568560f63cec8796e1c3a01" \
     --owner grenade@mozilla.com
   ```
3. deploy the mutated configurations from the deploy folder (use this command locally if you want to test configurations that you are unsure of. if you know the configurations are good, you can just commit and push an updated `/.deploy` folder and let the repo ci run the deploy step):
   ```
   cargo run -- deploy \
     #--taskcluster-root-url https://firefox-ci-tc.services.mozilla.com \
     --taskcluster-root-url https://stage.taskcluster.nonprod.cloudops.mozgcp.net \
     --worker-pool-pattern "^gecko-[1t]/[bt]-win(7-32|10-64|2012)-(beta|gpu-b)$"
   ```

### original python implementation

* production deployments are currently possible, but disabled.
* some configuration files are renamed with a `.yaml` (instead of `.yml`) extension. these are useful to have in the repo for templating purposes, but are **not deployed** on push.

#### installation

* clone this repository
  ```
  git clone https://github.com/mozilla-platform-ops/cloud-image-deploy.git
  ```
* install dependencies
  ```
  sudo apt install python3 || sudo dnf install python3
  pip3 install --user taskcluster pyyaml
  ```

#### configuration

* create or modify a worker pool definition in the config folder.
  * client definition config paths use the convention: `config/$environment/client/$clientId.yml` (where $clientId may contain path separators)
  * pool definition config paths use the convention: `config/$environment/pool/$domain/$pool.yml`
  * role definition config paths use the convention: `config/$environment/role/$roleId.yml` (where $roleId may contain path separators)
  ```

#### configuration (for running locally)
these steps only apply to running cloud-image-deploy locally. if you push modified (client/pool/role) configurations (using path conventions), they will deploy.

* modify `config/deploy-on-push.yml` to include only the pool definitions you wish to update. **take extra care here**. changes here can overwrite other peoples efforts and modify production worker pools. take care to only include pool definitions you are responsible for and remove or comment all other pool ids.
* create taskcluster clients for each environment you will deploy to
  * [production](https://firefox-ci-tc.services.mozilla.com/auth/clients)
  * [staging](https://stage.taskcluster.nonprod.cloudops.mozgcp.net/auth/clients)
* add scopes to the new clients
  * required client scopes include:
      ```
      worker-manager:manage-worker-pool:pool-id(s)
      worker-manager:provider:provider-id(s)
      ```
  * for example:
      ```
      worker-manager:manage-worker-pool:gecko-1/win*
      worker-manager:manage-worker-pool:gecko-1/b-win*
      worker-manager:manage-worker-pool:gecko-2/win*
      worker-manager:manage-worker-pool:gecko-2/b-win*
      worker-manager:manage-worker-pool:gecko-3/win*
      worker-manager:manage-worker-pool:gecko-3/b-win*
      worker-manager:manage-worker-pool:gecko-t/win*
      worker-manager:provider:az*
      ```
* create `config/taskcluster-client-options.yml` using `config/taskcluster-client-options-example.yml` as a template

#### usage (for running locally)
```bash
# obtain credentials using taskcluster-cli
set_session_credentials.sh
# deploy configured clients, pools, roles
python3 ci/deploy-on-push.py
```
