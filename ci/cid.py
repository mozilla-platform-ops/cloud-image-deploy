import taskcluster.exceptions
import yaml
from datetime import datetime, timedelta
from termcolor import colored


def updateClient(authClient, configPath, clientId):
    with open(configPath, 'r') as stream:
        payload = yaml.safe_load(stream)
        client = None
        try:
            client = authClient.client(clientId=clientId)
            print(colored('trace: client {} existence detected'.format(clientId), 'cyan'))
        except taskcluster.exceptions.TaskclusterRestFailure as tcRestFailure:
            if tcRestFailure.status_code == 404:
                client = None
                print(colored('trace: client {} absence detected'.format(clientId), 'cyan'))
            else:
                raise
        if client:
            try:
                authClient.updateClient(clientId, payload)
                print(colored('info: client {} updated'.format(clientId), 'yellow'))
            except taskcluster.exceptions.TaskclusterRestFailure as exception:
                print(colored('error: client {} update failed with status code {}'.format(clientId, exception.status_code), 'red'))
                raise
        else:
            try:
                authClient.createClient(clientId, payload)
                print(colored('info: client {} created'.format(clientId), 'yellow'))
            except taskcluster.exceptions.TaskclusterRestFailure as exception:
                print(colored('error: client {} create failed with status code {}'.format(clientId, exception.status_code), 'red'))
                raise

def updateRole(authClient, configPath, roleId):
    with open(configPath, 'r') as stream:
        payload = yaml.safe_load(stream)
        role = None
        try:
            role = authClient.role(roleId=roleId)
            print(colored('trace: role {} existence detected'.format(roleId), 'cyan'))
        except taskcluster.exceptions.TaskclusterRestFailure as tcRestFailure:
            if tcRestFailure.status_code == 404:
                role = None
                print(colored('trace: role {} absence detected'.format(roleId), 'cyan'))
            else:
                raise
        if role:
            try:
                authClient.updateRole(roleId, payload)
                print(colored('info: role {} updated'.format(roleId), 'yellow'))
            except taskcluster.exceptions.TaskclusterRestFailure as exception:
                print(colored('error: role {} update failed with status code {}'.format(roleId, exception.status_code), 'red'))
                raise
        else:
            try:
                authClient.createRole(roleId, payload)
                print(colored('info: role {} created'.format(roleId), 'yellow'))
            except taskcluster.exceptions.TaskclusterRestFailure as exception:
                print(colored('error: role {} create failed with status code {}'.format(roleId, exception.status_code), 'red'))
                raise

def updateWorkerPool(workerManagerClient, configPath, workerPoolId):
    with open(configPath, 'r') as stream:
        payload = yaml.safe_load(stream)
        try:
            workerManagerClient.workerPool(workerPoolId=workerPoolId)
            print(colored('trace: worker pool {} existence detected'.format(workerPoolId), 'cyan'))
            workerManagerClient.updateWorkerPool(workerPoolId, payload)
            print(colored('info: worker pool {} updated'.format(workerPoolId), 'yellow'))
        except taskcluster.exceptions.TaskclusterRestFailure as updateException:
            if updateException.status_code == 404:
                print(colored('trace: worker pool {} absence detected'.format(workerPoolId), 'cyan'))
                try:
                    workerManagerClient.createWorkerPool(workerPoolId, payload)
                    print(colored('info: worker pool {} created'.format(workerPoolId), 'yellow'))
                except taskcluster.exceptions.TaskclusterRestFailure as createException:
                    print(colored('error: worker pool {} create failed with status code {}'.format(workerPoolId, createException.status_code), 'red'))
                    raise
            else:
                print(colored('error: worker pool {} update failed with status code {}'.format(workerPoolId, updateException.status_code), 'red'))
                raise

def createTask(
        queueClient,
        taskId,
        taskName,
        taskDescription,
        provisioner,
        workerType,
        commands,
        env=None,
        image=None,
        priority='low',
        retries=0,
        retriggerOnExitCodes=[],
        dependencies=[],
        maxRunMinutes=10,
        features={},
        artifacts=[],
        osGroups=[],
        routes=[],
        scopes=[],
        taskGroupId=None):
    payload = {
        'created': '{}Z'.format(datetime.utcnow().isoformat()[:-3]),
        'deadline': '{}Z'.format(
            (datetime.utcnow() + timedelta(days=3)).isoformat()[:-3]),
        'dependencies': dependencies,
        'provisionerId': provisioner,
        'workerType': workerType,
        'priority': priority,
        'routes': routes,
        'scopes': scopes,
        'payload': {
            'maxRunTime': (maxRunMinutes * 60),
            'command': commands,
            'artifacts': artifacts if workerType.startswith('win') else {
                artifact['name']: {
                    'type': artifact['type'],
                    'path': artifact['path']
                } for artifact in artifacts
            },
            'features': features
        },
        'metadata': {
            'name': taskName,
            'description': taskDescription,
            'owner': 'grenade@mozilla.com',
            'source':
                'https://github.com/mozilla-platform-ops/cloud-image-builder'
        },
        'schedulerId': 'taskcluster-github'
    }
    if taskGroupId is not None:
        payload['taskGroupId'] = taskGroupId
    if env is not None:
        payload['payload']['env'] = env
    if image is not None:
        payload['payload']['image'] = image
    if osGroups:
        payload['payload']['osGroups'] = osGroups
    if retriggerOnExitCodes and retries > 0:
        payload['retries'] = retries
        payload['payload']['onExitStatus'] = {
            'retry': retriggerOnExitCodes
        }
    queueClient.createTask(taskId, payload)
    print(colored('info: task {} ({}: {}), created with priority: {}'.format(
        taskId, taskName, taskDescription, priority), 'yellow'))
