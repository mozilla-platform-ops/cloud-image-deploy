## cloud image deploy

this is a minimal implementation of a worker pool creator / updater.

it is useful for quickly and simply deploying one or more, new or updated worker pool definitions to multiple environments.

example use cases include:
* test updated cloud machine images before adding their id to ci-configuration
* test generic worker configuration changes
* stagger the rollout of a change whose impact at scale is unknown by modifying individual (regional/zonal) launch configurations one at a time instead of updating an entire pool (all availability zones) at the same time.

it is important to note that changes made by this application to taskcluster worker pool configurations are not permanent and will be overwritten any time ci-admin is run and the configurations here do not match the configurations in ci-configuration.