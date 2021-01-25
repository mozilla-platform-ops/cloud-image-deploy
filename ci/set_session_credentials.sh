#!/bin/bash

# this script creates temporary credentials for the staging and production taskcluster environments
# to be used by cloud-image-deploy/ci/deploy-on-push.py
# it does this by:
# * downloading the latest taskcluster cli to the users bin folder (~/bin) and symlinking 
# * opening a browser to authenticate this script's session against each environment
# * creating or updating a user client named ${USER}-dev to be used by cloud-image-deploy
# * storing the access token for ${USER}-dev in ${script_dir}/../config/${USER}-options.yml (used by deploy-on-push.py)

declare -A tc_environments
tc_environments[staging]=https://stage.taskcluster.nonprod.cloudops.mozgcp.net
tc_environments[production]=https://firefox-ci-tc.services.mozilla.com
gh_user=taskcluster
gh_repo=taskcluster
asset_name=taskcluster-linux-amd64
tc_client_id=${USER}-dev
tmp_dir=$(mktemp -d)
script_dir="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"

mkdir -p tmp_dir
curl -s https://api.github.com/repos/${gh_user}/${gh_repo}/releases/latest > ${tmp_dir}/latest.json
tag_name=$(jq --arg asset_name ${asset_name} -r '.tag_name' ${tmp_dir}/latest.json)
asset_url=$(jq --arg asset_name ${asset_name} -r '.assets[] | select(.name==$asset_name).browser_download_url' ${tmp_dir}/latest.json)

if [ ! -f ${HOME}/bin/${asset_name}${tag_name} ]; then
  curl -L -o ${HOME}/bin/${asset_name}${tag_name} ${asset_url}
  if [ ! -f ${HOME}/bin/${asset_name}${tag_name} ]; then
    echo "failed to download ${tmp_dir}/${asset_name}${tag_name} from ${asset_url}"
    exit
  else
    echo "downloaded ${HOME}/bin/${asset_name}${tag_name} from ${asset_url}"
    chmod +x ${HOME}/bin/${asset_name}${tag_name}
    ln -s ${HOME}/bin/${asset_name}${tag_name} ${HOME}/bin/${asset_name}
  fi
fi

for tc_environment in "${!tc_environments[@]}"; do
  export TASKCLUSTER_ROOT_URL=${tc_environments[${tc_environment}]}
  session_client_scopes=$(yq -r '.scopes | join(" --scope ")' ${script_dir}/../config/${tc_environment}/client/${tc_client_id}.yml)
  eval `${HOME}/bin/${asset_name} signin --name cloud-image-deploy --scope auth:create-client:${tc_client_id} --scope auth:update-client:${tc_client_id} --scope auth:reset-access-token:${tc_client_id} --scope ${session_client_scopes}`
  if ${HOME}/bin/${asset_name} api auth client ${tc_client_id} &> /dev/null; then
    echo "client ${tc_client_id} exists"
    yq --arg expires $(date -d '+1 day' -u '+%Y-%m-%dT%H:%M:%SZ') '. | .expires=$expires' ${script_dir}/../config/${tc_environment}/client/${tc_client_id}.yml | ${HOME}/bin/${asset_name} api auth updateClient ${tc_client_id} > ${tmp_dir}/update-${tc_environment}-client-${tc_client_id}-response.json
    ${HOME}/bin/${asset_name} api auth resetAccessToken ${tc_client_id} > ${tmp_dir}/reset-${tc_environment}-client-${tc_client_id}-response.json
    tc_client_access_token=$(jq -r '.accessToken' ${tmp_dir}/reset-${tc_environment}-client-${tc_client_id}-response.json)
    echo "client ${tc_client_id} updated with reset access token ${tc_client_access_token}"
    options_backup_path=${script_dir}/../config/${USER}-options-$(date '+%Y%m%d-%H%M%S').yml
    mv ${script_dir}/../config/${USER}-options.yml ${options_backup_path}
    yq --arg tc_client_id ${tc_client_id} --arg tc_access_token ${tc_client_access_token} --argjson tc_client_id_path "[\"${tc_environment}\",\"credentials\",\"clientId\"]" --argjson tc_access_token_path "[\"${tc_environment}\",\"credentials\",\"accessToken\"]" -y '. | getpath($tc_client_id_path)=$tc_client_id | getpath($tc_access_token_path)=$tc_access_token' ${options_backup_path} > ${script_dir}/../config/${USER}-options.yml
  else
    echo "client ${tc_client_id} does not exist"
    yq --arg expires $(date -d '+1 day' -u '+%Y-%m-%dT%H:%M:%SZ') '. | .expires=$expires' ${script_dir}/../config/${tc_environment}/client/${tc_client_id}.yml | ${HOME}/bin/${asset_name} api auth createClient ${tc_client_id} > ${tmp_dir}/create-${tc_environment}-client-${tc_client_id}-response.json
    tc_client_access_token=$(jq -r '.accessToken' ${tmp_dir}/create-${tc_environment}-client-${tc_client_id}-response.json)
    echo "client ${tc_client_id} created with access token ${tc_client_access_token}"
    options_backup_path=${script_dir}/../config/${USER}-options-$(date '+%Y%m%d-%H%M%S').yml
    mv ${script_dir}/../config/${USER}-options.yml ${options_backup_path}
    yq --arg tc_client_id ${tc_client_id} --arg tc_access_token ${tc_client_access_token} --argjson tc_client_id_path "[\"${tc_environment}\",\"credentials\",\"clientId\"]" --argjson tc_access_token_path "[\"${tc_environment}\",\"credentials\",\"accessToken\"]" -y '. | getpath($tc_client_id_path)=$tc_client_id | getpath($tc_access_token_path)=$tc_access_token' ${options_backup_path} > ${script_dir}/../config/${USER}-options.yml
  fi
done
rm -rf ${tmp_dir}
