#!/bin/bash
set -euo pipefail
export PATH=/usr/local/bin:/usr/bin:/bin
source "${XDG_CONFIG_HOME:-$HOME/.config}/sunrise-qol-runner/config.sh"
resources=${SUNRISE_RESOURCES:?}
export SUNRISE_RUNNER_REPOSITORY=${SUNRISE_RUNNER_REPOSITORY:?}
export SUNRISE_RUNNER_NAME=${SUNRISE_RUNNER_NAME:-sunrise-qol-private}
for file in facts.json gameplay-facts.json block-behaviours.json; do test -s "$resources/$file"; done
credential_file=$(mktemp "${XDG_RUNTIME_DIR:?}/sunrise-qol-runner.XXXXXX")
trap 'rm -f "$credential_file"' EXIT
chmod 600 "$credential_file"
registration_token=$(/usr/bin/gh api -X POST "repos/$SUNRISE_RUNNER_REPOSITORY/actions/runners/registration-token" --jq .token)
printf 'RUNNER_INPUT_TOKEN=%s\n' "$registration_token" > "$credential_file"
unset registration_token
/usr/bin/podman run --pull=never --rm --name sunrise-qol-runner --init --cap-drop=ALL --security-opt=no-new-privileges --pids-limit=2048 --memory=6g --memory-swap=8g --cpus=2 --env-file "$credential_file" --env SUNRISE_RUNNER_REPOSITORY --env SUNRISE_RUNNER_NAME --volume "$resources:/opt/sunrise-resources/26.3:ro" localhost/sunrise-qol-runner:latest
