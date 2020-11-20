import yaml
import taskcluster.exceptions
from datetime import datetime, timedelta


def updateClient(authClient, configPath, clientId):
    with open(configPath, 'r') as stream:
        payload = yaml.safe_load(stream)
        client = None
        try:
            client = authClient.client(clientId=clientId)
            print('info: client {} existence detected'.format(clientId))
        except taskcluster.exceptions.TaskclusterRestFailure as tcRestFailure:
            if tcRestFailure.status_code == 404:
                client = None
                print('info: client {} absence detected'.format(clientId))
            else:
                raise
        if client:
            authClient.updateClient(clientId, payload)
            print('info: client {} updated'.format(clientId))
        else:
            authClient.createClient(clientId, payload)
            print('info: client {} created'.format(clientId))

def updateRole(authClient, configPath, roleId):
    with open(configPath, 'r') as stream:
        payload = yaml.safe_load(stream)
        role = None
        try:
            role = authClient.role(roleId=roleId)
            print('info: role {} existence detected'.format(roleId))
        except taskcluster.exceptions.TaskclusterRestFailure as tcRestFailure:
            if tcRestFailure.status_code == 404:
                role = None
                print('info: role {} absence detected'.format(roleId))
            else:
                raise
        if role:
            authClient.updateRole(roleId, payload)
            print('info: role {} updated'.format(roleId))
        else:
            authClient.createRole(roleId, payload)
            print('info: role {} created'.format(roleId))

def updateWorkerPool(workerManagerClient, configPath, workerPoolId):
    with open(configPath, 'r') as stream:
        payload = yaml.safe_load(stream)
        try:
            workerManagerClient.workerPool(workerPoolId=workerPoolId)
            print('info: worker pool {} existence detected'.format(
                workerPoolId))
            workerManagerClient.updateWorkerPool(workerPoolId, payload)
            print('info: worker pool {} updated'.format(workerPoolId))
        except taskcluster.exceptions.TaskclusterRestFailure as tcRestFailure:
            if tcRestFailure.status_code == 404:
                print('info: worker pool {} absence detected'.format(
                    workerPoolId))
                workerManagerClient.createWorkerPool(workerPoolId, payload)
                print('info: worker pool {} created'.format(workerPoolId))
            else:
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
    print('info: task {} ({}: {}), created with priority: {}'.format(
        taskId, taskName, taskDescription, priority))
