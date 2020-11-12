import yaml
import os
import taskcluster
from cid import updateWorkerPool


basePath = os.path.abspath(os.path.dirname(__file__))


with open('{}/../config/taskcluster-client-options.yml'.format(basePath), 'r') as secretsStream:
    taskclusterOptions = yaml.safe_load(secretsStream)
    with open('{}/../config/deploy-on-push.yml'.format(basePath), 'r') as stream:
        deployConfig = yaml.safe_load(stream)
        for environment in deployConfig:
            workerManager = taskcluster.WorkerManager(taskclusterOptions[environment])
            print('taskcluster {} rootUrl: {}'.format(environment, taskclusterOptions[environment]['rootUrl']))
            for domain in deployConfig[environment]:
                for pool in deployConfig[environment][domain]:
                    poolId = '{}/{}'.format(domain, pool)
                    poolConfigPath = os.path.abspath('{}/../config/{}/{}/{}.yml'.format(basePath, environment, domain, pool))
                    print('- updating pool: {}/{} with {}'.format(environment, poolId, poolConfigPath))
                    updateWorkerPool(workerManager, poolConfigPath, poolId)
