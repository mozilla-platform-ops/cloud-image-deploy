import glob
import yaml
import os
import sys
import taskcluster
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
    image = 'rust:latest',
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
        #    'name': 'public/unattend.xml',
        #    'path': 'unattend.xml'
        #},
        #{
        #    'type': 'file',
        #    'name': 'public/image-bucket-resource.json',
        #    'path': 'image-bucket-resource.json'
        #}
    ],
    commands = [
        '/bin/bash',
        '--login',
        '-c',
        'git clone --quiet https://github.com/mozilla-platform-ops/cloud-image-deploy',
        '&& cd cloud-image-deploy',
        '&& git fetch',
        '&& git checkout {}'.format(commitSha),
        '&& git reset --hard {}'.format(commitSha),
        '&& cargo run'
    ],
    routes = [
        #'index.project.relops.cloud-image-deploy.{}.{}.revision.{}'.format(platform, key, commitSha),
        #'index.project.relops.cloud-image-deploy.{}.{}.latest'.format(platform, key)
    ],
    scopes = [
        #'secrets:get:project/relops/image-deploy/dev'
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




      (tasks_for == "github-push"):
        taskId:
          $eval: as_slugid("rusty")
        deadline:
          $fromNow: 1 day
        provisionerId: relops-3
        workerType: decision
        routes:
          - index.project.relops.${event.repository.name}.rusty.revision.${event.after}
          - index.project.relops.${event.repository.name}.rusty.latest
        scopes:
          - assume:repo:github.com/${event.organization.login}/${event.repository.name}:branch:${event.repository.default_branch}
          - queue:scheduler-id:taskcluster-github
        extra:
          github:
            env: true
          data:
            base:
              sha: ${event.before}
              user:
                login: ${event.sender.login}
            head:
              sha: ${event.after}
              user:
                email: ${event.pusher.email}
        metadata:
          name: '00 :: generate ci configuration patches'
          description: |+
              ## this is a work in progress
              the intention here is to generate patches for ci-config containing updated worker pool configurations
          owner: ${event.pusher.email}
          source: ${event.repository.html_url}
        payload:
          maxRunTime: 600
          image: rust:latest
          features:
            taskclusterProxy: true
          env:
            GITHUB_HEAD_SHA: ${event.after}
          command:
            - /bin/bash
            - '--login'
            - '-c'
            - >-
              git clone --quiet ${event.repository.clone_url}
              && cd ${event.repository.name}
              && git fetch
              && git checkout ${head_rev}
              && git reset --hard ${event.after}
              && cargo run