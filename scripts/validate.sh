#!/usr/bin/env bash
# Runs the assignment's validation flow (steps 1-11) against a running API.
# Usage: ./scripts/validate.sh [API_BASE_URL]   (default http://localhost:8080)
# Needs curl plus jq or node for JSON parsing.
set -euo pipefail

API="${1:-${API_BASE_URL:-http://localhost:8080}}"

json() { # json '<expr>'  reads JSON on stdin; expr is a jq path like .a.b or .a[0]
  if command -v jq >/dev/null 2>&1; then jq -r "$1"; else
    node -e 'let s="";process.stdin.on("data",d=>s+=d).on("end",()=>{const v=eval("JSON.parse(s)"+process.argv[1].replace(/^\./,"?."));console.log(typeof v==="object"?JSON.stringify(v,null,2):v)})' "$1"
  fi
}
pretty() { if command -v jq >/dev/null 2>&1; then jq .; else node -e 'let s="";process.stdin.on("data",d=>s+=d).on("end",()=>console.log(JSON.stringify(JSON.parse(s),null,2)))'; fi; }

post() { curl -sS -X POST "$API$1" -H 'Content-Type: application/json' ${3:+-H "Authorization: Bearer $3"} -d "$2"; }
get()  { curl -sS "$API$1" ${2:+-H "Authorization: Bearer $2"}; }
step() { printf '\n\033[1m==> %s\033[0m\n' "$*"; }
need() { # need <name> <value>: abort the run if a step produced no value
  if [ -z "$2" ] || [ "$2" = "null" ] || [ "$2" = "undefined" ]; then echo "FAILED: no $1 returned" >&2; exit 1; fi
}

login() { # login <email> <password> -> prints JWT
  local challenge code
  local token
  challenge=$(post /auth/login "{\"email\":\"$1\",\"password\":\"$2\"}" | json .login_challenge_id); need login_challenge_id "$challenge"
  code=$(get "/dev/email-logs/latest?email=$1" | json .code); need code "$code"
  echo "    challenge=$challenge code=$code (from dev mailbox)" >&2
  token=$(post /auth/verify-2fa "{\"login_challenge_id\":\"$challenge\",\"code\":\"$code\"}" | json .access_token); need access_token "$token"
  echo "$token"
}

printf 'Waiting for %s/health ' "$API"
for _ in $(seq 1 60); do curl -fsS "$API/health" >/dev/null 2>&1 && break; printf .; sleep 2; done
curl -fsS "$API/health" >/dev/null || { echo " API not reachable"; exit 1; }
echo " ok"

step "0. Reset dev data (keeps users) for a repeatable run"
post /dev/reset '{}' | json .message

step "1. Seed Admin and James Bond"
post /seed/users '{}' | json '.users' | grep -E '"(email|role)"'

step "2-4. Admin: login -> 2FA code from dev mailbox -> verify -> JWT"
login_response=$(post /auth/login '{"email":"admin@example.com","password":"Admin@12345"}')
echo "    login response has access_token? $(echo "$login_response" | grep -q access_token && echo yes || echo no)"
ADMIN=$(login admin@example.com 'Admin@12345') || exit 1
echo "    admin JWT: ${ADMIN:0:24}..."

step "5. Admin creates exactly 5 tasks"
IDS=()
for spec in "Infiltrate SPECTRE HQ:high" "Recover the Lektor:medium" "Brief M on Blofeld:low" \
            "Service the Aston Martin:medium" "Collect gadgets from Q:low"; do
  id=$(post /tasks "{\"title\":\"${spec%%:*}\",\"priority\":\"${spec##*:}\"}" "$ADMIN" | json .id)
  need "task id" "$id"; IDS+=("$id"); echo "    created ${spec%%:*} ($id)"
done

step "6. Assign exactly 3 tasks to James Bond"
post /tasks/assign "{\"task_ids\":[\"${IDS[0]}\",\"${IDS[1]}\",\"${IDS[2]}\"],\"assignee_email\":\"jamesbond@example.com\"}" "$ADMIN" | json .updated_count | sed 's/^/    updated_count=/'

step "7-8. James Bond: login -> 2FA -> JWT"
JAMES=$(login jamesbond@example.com 'JamesBond@007') || exit 1
echo "    james JWT: ${JAMES:0:24}..."

step "9. James Bond tries to create a task (expect 403)"
status=$(curl -sS -o /tmp/james_create.json -w '%{http_code}' -X POST "$API/tasks" \
  -H 'Content-Type: application/json' -H "Authorization: Bearer $JAMES" -d '{"title":"License to create"}')
echo "    HTTP $status $(cat /tmp/james_create.json)"

step "10. GET /tasks/view-my-tasks (first call: from database)"
get /tasks/view-my-tasks "$JAMES" | pretty

step "11. GET /tasks/view-my-tasks again (from cache)"
get /tasks/view-my-tasks "$JAMES" | pretty
