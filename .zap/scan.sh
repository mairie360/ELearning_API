#!/bin/bash
# Entry point of the `security-scan` service (docker-compose-security.yml): runs zap-api-scan.py
# with the arguments of the compose `command`, plus a replacer that injects the admin JWT on every
# request so the authenticated routes are scanned instead of answering 401.
#
# ADMIN_JWT is the EdDSA token (sub=1, valid 2 hours) that security_test.sh signs with the run's
# Ed25519 key like Core does (test_secrets.sh, MAIR-680): the API only holds the public key and
# refuses every HS256 token. No key or token is committed (MAIR-428).
set -euo pipefail

TOKEN="${ADMIN_JWT:?must be the admin JWT signed by security_test.sh}"

# -silent: no add-on update check or telemetry at daemon start (it can hang the scan with
#   "Failed to connect to ZAP after 600 seconds").
# -z is split with shlex: 'Bearer <jwt>' must stay quoted, otherwise ZAP only receives "Bearer"
#   and the token is read as a file name.
exec zap-api-scan.py "$@" -z "-silent \
-config replacer.full_list(0).description=auth \
-config replacer.full_list(0).enabled=true \
-config replacer.full_list(0).matchtype=REQ_HEADER \
-config replacer.full_list(0).matchstr=Authorization \
-config replacer.full_list(0).regex=false \
-config replacer.full_list(0).replacement='Bearer ${TOKEN}'"
