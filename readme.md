## cloud image deploy

this is a minimal implementation of a worker pool creator / updater.

it is useful for quickly and simply deploying one or more, new or updated worker pool definitions to multiple environments.

example use cases include:
* test updated cloud machine images before adding their id to ci-configuration
* test generic worker configuration changes
* stagger the rollout of a change whose impact at scale is unknown by modifying individual (regional/zonal) launch configurations one at a time instead of updating an entire pool (all availability zones) at the same time.

it is important to note that changes made by this application to taskcluster worker pool configurations are not permanent and will be overwritten any time ci-admin is run and the configurations here do not match the configurations in ci-configuration.

### installation

* clone this repository
  ```
  git clone https://github.com/mozilla-platform-ops/cloud-image-deploy.git
  ```
* install dependencies
  ```
  sudo apt install python3 || sudo dnf install python3
  pip3 install --user taskcluster pyyaml
  ```

### configure

* create or modify a worker pool definition in the config folder. pool definition config paths use the convention: `config/$environment/$domain/$pool.yml`
* modify `config/deploy-on-push.yml` to include only the pool definitions you wish to update. **take extra care here**. changes here can overwrite other peoples efforts and modify production worker pools. take care to only include pool definitions you are responsible for and remove or comment all other pool ids.
* create taskcluster clients for each environment you will deploy to
  * [production](https://firefox-ci-tc.services.mozilla.com/auth/clients)
  * [staging](https://stage.taskcluster.nonprod.cloudops.mozgcp.net/auth/clients)
* create `config/taskcluster-client-options`.yml using `config/taskcluster-client-options-example.yml` as a template

### usage
```
python3 ci/deploy-on-push.py
```
