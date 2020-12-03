import yaml
import os
import taskcluster
from cid import updateClient, updateRole, updateWorkerPool


basePath = os.path.abspath(os.path.dirname(__file__))

secretsOverridePath = '{}/../config/{}-options.yml'.format(basePath, os.getenv('USER'))
secretsPath = secretsOverridePath if os.path.isfile(secretsOverridePath) else '{}/../config/taskcluster-client-options.yml'.format(basePath)
with open(secretsPath, 'r') as secretsStream:
    taskclusterOptions = yaml.safe_load(secretsStream)
    with open('{}/../config/deploy-on-push.yml'.format(basePath), 'r') as deployStream:
        deployConfig = yaml.safe_load(deployStream)
        for environment in deployConfig:
            print('taskcluster {} rootUrl: {}'.format(environment, taskclusterOptions[environment]['rootUrl']))

            authClient = taskcluster.Auth(taskclusterOptions[environment])
            workerManagerClient = taskcluster.WorkerManager(taskclusterOptions[environment])

            # clients
            for clientId in deployConfig[environment]['client']:
                clientConfigPath = os.path.abspath('{}/../config/{}/client/{}.yml'.format(basePath, environment, clientId))
                print('- updating client: {}/{} with {}'.format(environment, clientId, clientConfigPath))
                updateClient(authClient, clientConfigPath, clientId)
            # roles
            for roleId in deployConfig[environment]['role']:
                roleConfigPath = os.path.abspath('{}/../config/{}/role/{}.yml'.format(basePath, environment, roleId))
                print('- updating role: {}/{} with {}'.format(environment, roleId, roleConfigPath))
                updateRole(authClient, roleConfigPath, roleId)

            # pools
            for domain in deployConfig[environment]['pool']:
                for pool in deployConfig[environment]['pool'][domain]:
                    poolId = '{}/{}'.format(domain, pool)
                    poolConfigPath = os.path.abspath('{}/../config/{}/pool/{}/{}.yml'.format(basePath, environment, domain, pool))
                    print('- updating pool: {}/{} with {}'.format(environment, poolId, poolConfigPath))
                    updateWorkerPool(workerManagerClient, poolConfigPath, poolId)

