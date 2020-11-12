import yaml
import taskcluster.exceptions
from datetime import datetime, timedelta


def updateRole(auth, configPath, roleId):
    with open(configPath, 'r') as stream:
        payload = yaml.safe_load(stream)
        role = None
        try:
            role = auth.role(roleId=roleId)
            print('info: role {} existence detected'.format(roleId))
        except taskcluster.exceptions.TaskclusterRestFailure as tcRestFailure:
            if tcRestFailure.status_code == 404:
                role = None
                print('info: role {} absence detected'.format(roleId))
            else:
                raise

        if role:
            auth.updateRole(roleId, payload)
            print('info: role {} updated'.format(roleId))
        else:
            auth.createRole(roleId, payload)
            print('info: role {} created'.format(roleId))


def updateWorkerPool(workerManager, configPath, workerPoolId):
    with open(configPath, 'r') as stream:
        payload = yaml.safe_load(stream)
        try:
            workerManager.workerPool(workerPoolId=workerPoolId)
            print('info: worker pool {} existence detected'.format(
                workerPoolId))
            workerManager.updateWorkerPool(workerPoolId, payload)
            print('info: worker pool {} updated'.format(workerPoolId))
        except taskcluster.exceptions.TaskclusterRestFailure as tcRestFailure:
            if tcRestFailure.status_code == 404:
                print('info: worker pool {} absence detected'.format(
                    workerPoolId))
                workerManager.createWorkerPool(workerPoolId, payload)
                print('info: worker pool {} created'.format(workerPoolId))
            else:
                raise


def createTask(
        queue,
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

    queue.createTask(taskId, payload)
    print('info: task {} ({}: {}), created with priority: {}'.format(
        taskId, taskName, taskDescription, priority))
