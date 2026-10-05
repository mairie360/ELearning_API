#!/bin/bash
# Entry point of the `security-scan` service (docker-compose-security.yml): signs a short-lived
# admin JWT with the stack's JWT_SECRET, then runs zap-api-scan.py with the arguments of the
# compose `command`, plus a replacer that injects the token on every request so the
# authenticated routes are scanned instead of answering 401.
#
# No token is committed (MAIR-428): the repository is public, and a static token signed with the
# test secret would be admin wherever that secret were ever reused, until its `exp`.
set -euo pipefail

: "${JWT_SECRET:?must be the JWT_SECRET of the elearning service}"

# HS256, sub=1 (the Admin seeded by liquibase), valid for 2 hours.
TOKEN=$(python3 -c '
import base64, hashlib, hmac, json, os, time

def b64(data):
    return base64.urlsafe_b64encode(data).rstrip(b"=").decode()

header = b64(json.dumps({"alg": "HS256", "typ": "JWT"}).encode())
claims = {"sub": "1", "role": "admin", "exp": int(time.time()) + 2 * 3600}
payload = b64(json.dumps(claims).encode())
signature = hmac.new(
    os.environ["JWT_SECRET"].encode(), f"{header}.{payload}".encode(), hashlib.sha256
).digest()
print(f"{header}.{payload}.{b64(signature)}")
')

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
