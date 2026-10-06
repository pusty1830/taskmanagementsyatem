# Runs the assignment's validation flow (steps 1-11) against a running API.
# Usage: powershell -ExecutionPolicy Bypass -File scripts\validate.ps1 [-Api http://localhost:8080]
param([string]$Api = $(if ($env:API_BASE_URL) { $env:API_BASE_URL } else { 'http://localhost:8080' }))
$ErrorActionPreference = 'Stop'

function Post($Path, $Body, $Token) {
    $headers = @{}
    if ($Token) { $headers.Authorization = "Bearer $Token" }
    Invoke-RestMethod -Method Post -Uri "$Api$Path" -Headers $headers -ContentType 'application/json' -Body ($Body | ConvertTo-Json -Depth 5)
}
function Get-Api($Path, $Token) {
    $headers = @{}
    if ($Token) { $headers.Authorization = "Bearer $Token" }
    Invoke-RestMethod -Method Get -Uri "$Api$Path" -Headers $headers
}
function Step($Text) { Write-Host "`n==> $Text" -ForegroundColor Cyan }
function Login($Email, $Password) {
    $challenge = (Post '/auth/login' @{ email = $Email; password = $Password }).login_challenge_id
    $code = (Get-Api "/dev/email-logs/latest?email=$Email").code
    Write-Host "    challenge=$challenge code=$code (from dev mailbox)"
    (Post '/auth/verify-2fa' @{ login_challenge_id = $challenge; code = $code }).access_token
}

Step '0. Reset dev data (keeps users) for a repeatable run'
(Post '/dev/reset' @{}).message

Step '1. Seed Admin and James Bond'
(Post '/seed/users' @{}).users | Format-Table email, role

Step '2-4. Admin: login -> 2FA code from dev mailbox -> verify -> JWT'
$first = Post '/auth/login' @{ email = 'admin@example.com'; password = 'Admin@12345' }
Write-Host "    login response has access_token? $([bool]$first.access_token)"
$admin = Login 'admin@example.com' 'Admin@12345'
Write-Host "    admin JWT: $($admin.Substring(0, 24))..."

Step '5. Admin creates exactly 5 tasks'
$ids = @()
foreach ($t in @(@('Infiltrate SPECTRE HQ', 'high'), @('Recover the Lektor', 'medium'), @('Brief M on Blofeld', 'low'),
                 @('Service the Aston Martin', 'medium'), @('Collect gadgets from Q', 'low'))) {
    $task = Post '/tasks' @{ title = $t[0]; priority = $t[1] } $admin
    $ids += $task.id
    Write-Host "    created $($t[0]) ($($task.id))"
}

Step '6. Assign exactly 3 tasks to James Bond'
$res = Post '/tasks/assign' @{ task_ids = @($ids[0], $ids[1], $ids[2]); assignee_email = 'jamesbond@example.com' } $admin
Write-Host "    updated_count=$($res.updated_count)"

Step '7-8. James Bond: login -> 2FA -> JWT'
$james = Login 'jamesbond@example.com' 'JamesBond@007'
Write-Host "    james JWT: $($james.Substring(0, 24))..."

Step '9. James Bond tries to create a task (expect 403)'
try {
    Post '/tasks' @{ title = 'License to create' } $james | Out-Null
    Write-Host '    UNEXPECTED: task was created' -ForegroundColor Red
} catch {
    Write-Host "    HTTP $([int]$_.Exception.Response.StatusCode) $($_.ErrorDetails.Message)"
}

Step '10. GET /tasks/view-my-tasks (first call: from database)'
Get-Api '/tasks/view-my-tasks' $james | ConvertTo-Json -Depth 6

Step '11. GET /tasks/view-my-tasks again (from cache)'
Get-Api '/tasks/view-my-tasks' $james | ConvertTo-Json -Depth 6
