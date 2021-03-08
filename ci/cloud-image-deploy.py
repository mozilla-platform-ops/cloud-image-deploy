import glob
import os
import slugid
import sys
import taskcluster
import yaml
from cid import createTask, updateClient, updateRole, updateWorkerPool
from termcolor import colored

environment = 'staging' if 'stage' in os.environ.get('TASKCLUSTER_ROOT_URL', '') else 'production'
#if environment != 'staging':
#    print('info: skipping non staging deployments')
#    quit()

basePath = os.path.abspath(os.path.dirname(__file__))
cfgPath = os.path.abspath(os.path.join(basePath, '../config', environment))
taskclusterOptions = { 'rootUrl': os.environ['TASKCLUSTER_PROXY_URL'] }

print('- taskcluster {} deployment'.format(environment))
print('  - rootUrl: {}'.format(os.environ.get('TASKCLUSTER_ROOT_URL', '')))
print('  - proxyUrl: {}'.format(os.environ.get('TASKCLUSTER_PROXY_URL', '')))

authClient = taskcluster.Auth(taskclusterOptions)
queueClient = taskcluster.Queue(taskclusterOptions)
workerManagerClient = taskcluster.WorkerManager(taskclusterOptions)

commitSha = os.getenv('GITHUB_HEAD_SHA')
taskGroupId = os.getenv('TASK_ID')

createTask(
    queueClient = queueClient,
    image = 'grenade/opencloudconfig@sha256:26b6d65e4a8136c97ea1805fdbf947d80a843178f7ba1fd6501d249ac887d671',
    taskId = slugid.nice(),
    taskName = '01 :: generate ci configuration patches (wip)',
    taskDescription = 'the intention here is to generate patches for ci-config containing updated worker pool configurations',
    maxRunMinutes = 10,
    retries = 2,
    provisioner = 'relops-3',
    workerType = 'decision',
    priority = 'high',
    features = {
        'taskclusterProxy': True
    },
    artifacts = [
        #{
        #    'type': 'file',
        #    'name': 'public/artifact.json',
        #    'path': 'artifact.json'
        #}
    ],
    env = {
        'GITHUB_HEAD_SHA': commitSha,
        'TASK_GROUP_ID': taskGroupId,
    },
    commands = [
        '/bin/bash',
        '--login',
        '-c',
        ' && '.join([
            'git clone --quiet https://github.com/mozilla-platform-ops/cloud-image-deploy',
            'cd cloud-image-deploy',
            'git fetch',
            'git checkout {}'.format(commitSha),
            'git reset --hard {}'.format(commitSha),

            # todo: move the build to a task that only runs when rust source has changed,
            # instead of building, download the executable from the last successful build task run
            'cargo build --verbose',
            'cargo test --verbose',

            #' '.join([# snapshot command:
            #    'cargo run --',
            #    'snapshot',
            #    '--client-pattern "relops"',
            #    '--role-pattern "relops|mozilla-platform-ops|OpenCloudConfig|worker-pool:.*win.*|workerPool.*win.*"',
            #    '--worker-pool-pattern "^(relops.*|(gecko|mpd001)-[1-3t]/([bt]-)?win.*)$"',
            #]),

            ' '.join([# deploy command:
                'cargo run --',
                'deploy',
                '--worker-pool-pattern "^(relops.*|(gecko|mpd001)-[1-3t]/([bt]-)?win.*)$"',
            ]),
        ])
    ],
    routes = [
        #'index.project.relops.cloud-image-deploy.{}.{}.revision.{}'.format(platform, key, commitSha),
        #'index.project.relops.cloud-image-deploy.{}.{}.latest'.format(platform, key)
    ],
    scopes = [
        'queue:create-task:highest:gecko-1/b-win*',
        'queue:create-task:highest:gecko-1/win*',
        'queue:create-task:highest:gecko-t/t-win*',
        'queue:create-task:highest:gecko-t/win*',
        'queue:scheduler-id:taskcluster-github',
        'worker-manager:manage-worker-pool:gecko-1/b-win*',
        'worker-manager:manage-worker-pool:gecko-1/win*',
        'worker-manager:manage-worker-pool:gecko-t/t-win*',
        'worker-manager:manage-worker-pool:gecko-t/win*',
        'worker-manager:provider:aws',
        'worker-manager:provider:azure',
    ],
    taskGroupId = taskGroupId
)

# clients
for clientConfigPath in glob.glob(os.path.join(cfgPath, 'client', '**', '*.yml'), recursive = True):
    clientId = clientConfigPath.replace(os.path.join(cfgPath, 'client'), '').replace('.yml', '').lstrip('/')
    print('- updating client: {}/{} with {}'.format(environment, clientId, clientConfigPath))
    try:
        updateClient(authClient, clientConfigPath, clientId)
    except taskcluster.exceptions.TaskclusterRestFailure:
        print(colored('  - client update failed: taskcluster rest failure', 'red'))
        # todo: check for scope exceptions and notify missing scopes
    except:
        print(colored('  - client update failed: {}'.format(sys.exc_info()[0]), 'red'))
# roles
for roleConfigPath in glob.glob(os.path.join(cfgPath, 'role', '**', '*.yml'), recursive = True):
    roleId = roleConfigPath.replace(os.path.join(cfgPath, 'role'), '').replace('.yml', '').lstrip('/')
    print('- updating role: {}/{} with {}'.format(environment, roleId, roleConfigPath))
    try:
        updateRole(authClient, roleConfigPath, roleId)
    except taskcluster.exceptions.TaskclusterRestFailure:
        print(colored('  - role update failed: taskcluster rest failure', 'red'))
        # todo: check for scope exceptions and notify missing scopes
    except:
        print(colored('  - role update failed: {}'.format(sys.exc_info()[0]), 'red'))

# pools
for poolConfigPath in glob.glob(os.path.join(cfgPath, 'pool', '**', '*.yml'), recursive = True):
    poolId = poolConfigPath.replace(os.path.join(cfgPath, 'pool'), '').replace('.yml', '').lstrip('/')
    print('- updating pool: {}/{} with {}'.format(environment, poolId, poolConfigPath))
    try:
        updateWorkerPool(workerManagerClient, poolConfigPath, poolId)
    except taskcluster.exceptions.TaskclusterRestFailure:
        print(colored('  - pool update failed: taskcluster rest failure', 'red'))
        raise
    except:
        print(colored('  - pool update failed: {}'.format(sys.exc_info()[0]), 'red'))
