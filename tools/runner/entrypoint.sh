#!/bin/bash
set -euo pipefail
cd /runner
./config.sh --unattended --ephemeral --url "https://github.com/${SUNRISE_RUNNER_REPOSITORY:?}" --name "${SUNRISE_RUNNER_NAME:-sunrise-qol-private}" --labels sunrise-qol-private --work /runner/_work --replace
unset RUNNER_INPUT_TOKEN
exec ./run.sh
