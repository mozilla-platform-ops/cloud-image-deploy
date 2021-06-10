#!/bin/bash

cd ~/git/mozilla-platform-ops/cloud-image-deploy

cargo run -- mutate \
  --taskcluster-root-url https://firefox-ci-tc.services.mozilla.com \
  --worker-pool-pattern "^gecko-t/t-win10-64-(beta|gpu-b)$" \
  --build-sha "mozilla-releng/OpenCloudConfig/abe6f81b77fc62165e523ba263e343ab4fa59f50" \
  --owner grenade@mozilla.com
