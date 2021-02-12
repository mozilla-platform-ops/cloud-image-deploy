import yaml
import os
import sys
import taskcluster
from cid import updateClient, updateRole, updateWorkerPool
from termcolor import colored


basePath = os.path.abspath(os.path.dirname(__file__))

secretsOverridePath = '{}/../config/{}-options.yml'.format(basePath, os.getenv('USER'))
secretsPath = secretsOverridePath if os.path.isfile(secretsOverridePath) else '{}/../config/taskcluster-client-options.yml'.format(basePath)
with open(secretsPath, 'r') as secretsStream:
    taskclusterOptions = yaml.safe_load(secretsStream)
    with open('{}/../config/deploy-on-demand.yml'.format(basePath), 'r') as deployStream:
        deployConfig = yaml.safe_load(deployStream)
        for environment in deployConfig:
            print('taskcluster {} rootUrl: {}'.format(environment, taskclusterOptions[environment]['rootUrl']))

            authClient = taskcluster.Auth(taskclusterOptions[environment])
            workerManagerClient = taskcluster.WorkerManager(taskclusterOptions[environment])

            # clients
            for clientId in deployConfig[environment]['client']:
                clientConfigRelativePath = '{}/../config/{}/client/{}.yml'.format(basePath, environment, clientId)
                if not os.path.isfile(clientConfigRelativePath):
                    clientConfigRelativePath = '{}/../config/{}/client/{}.yaml'.format(basePath, environment, clientId)
                clientConfigPath = os.path.abspath(clientConfigRelativePath)
                print('- updating client: {}/{} with {}'.format(environment, clientId, clientConfigPath))
                try:
                    updateClient(authClient, clientConfigPath, clientId)
                except taskcluster.exceptions.TaskclusterRestFailure:
                    print(colored('  - client update failed: taskcluster rest failure', 'red'))
                    # todo: check for scope exceptions and notify / correct missing scopes
                except:
                    print(colored('  - client update failed: {}'.format(sys.exc_info()[0]), 'red'))
            # roles
            for roleId in deployConfig[environment]['role']:
                roleConfigRelativePath = '{}/../config/{}/role/{}.yml'.format(basePath, environment, roleId)
                if not os.path.isfile(roleConfigRelativePath):
                    roleConfigRelativePath = '{}/../config/{}/role/{}.yaml'.format(basePath, environment, roleId)
                roleConfigPath = os.path.abspath(roleConfigRelativePath)
                print('- updating role: {}/{} with {}'.format(environment, roleId, roleConfigPath))
                try:
                    updateRole(authClient, roleConfigPath, roleId)
                except taskcluster.exceptions.TaskclusterRestFailure:
                    print(colored('  - role update failed: taskcluster rest failure', 'red'))
                    # todo: check for scope exceptions and notify / correct missing scopes
                    raise
                except:
                    print(colored('  - role update failed: {}'.format(sys.exc_info()[0]), 'red'))

            # pools
            for domain in deployConfig[environment]['pool']:
                for pool in deployConfig[environment]['pool'][domain]:
                    poolId = '{}/{}'.format(domain, pool)
                    poolConfigRelativePath = '{}/../config/{}/pool/{}/{}.yml'.format(basePath, environment, domain, pool)
                    if not os.path.isfile(poolConfigRelativePath):
                        poolConfigRelativePath = '{}/../config/{}/pool/{}/{}.yaml'.format(basePath, environment, domain, pool)
                    poolConfigPath = os.path.abspath(poolConfigRelativePath)
                    print('- updating pool: {}/{} with {}'.format(environment, poolId, poolConfigPath))
                    try:
                        updateWorkerPool(workerManagerClient, poolConfigPath, poolId)
                    except taskcluster.exceptions.TaskclusterRestFailure:
                        print(colored('  - pool update failed: taskcluster rest failure', 'red'))
                        # todo: check for scope exceptions and notify / correct missing scopes
                    except:
                        print(colored('  - pool update failed: {}'.format(sys.exc_info()[0]), 'red'))

