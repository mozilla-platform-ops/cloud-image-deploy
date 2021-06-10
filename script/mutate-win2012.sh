#!/bin/bash

cd ~/git/mozilla-platform-ops/cloud-image-deploy

cargo run -- mutate \
  --taskcluster-root-url https://firefox-ci-tc.services.mozilla.com \
  --worker-pool-pattern "^gecko-[13]/b-win2012(-beta)?$" \
  --build-sha "mozilla-releng/OpenCloudConfig/dad85e2387a3100103c39543fe6675a2ca94421b" \
  --owner grenade@mozilla.com
