# 7. Frontend (React 19 + TypeScript + Vite)

## Screens and routes

| Route | Screen | Access | Contents |
|---|---|---|---|
| `/login` | **LoginPage** | Public | Step 1: email and password. Step 2: 6-digit code. A dev-only **"Fetch code from dev mailbox"** button calls `/dev/email-logs/latest` and fills in the code |
| `/admin` | **AdminPage** | role = admin | **Create task** form (title, description, priority). **All tasks** table with checkboxes. **Assign panel**: staff dropdown (from `/users?role=staff`) and an "Assign selected" button |
| `/my-tasks` | **MyTasksPage** | Any logged-in user | The user's assigned tasks, a **cache badge** (`cache: HIT` / `MISS`), a total count, and a **Refresh** button that calls the API again |
| | *"Try creating a task"* panel on `/my-tasks` for staff | staff | Reuses `TaskForm`. Submitting it as James shows: **"⛔ Forbidden: only admins can create tasks (403)"** |

After 2FA, an admin lands on `/admin` and staff land on `/my-tasks`. The route guard (`ProtectedRoute`) is only for UX. The API enforces every rule, which is exactly what the staff "try creating" panel demonstrates.

## Structure

```
src/
├── api/
│   ├── client.ts      # request<T>(method, path, body?) — base URL, JSON, Bearer header, ApiError
│   ├── types.ts       # TaskDto, MyTasksResponse, LoginResponse, … (mirror backend DTOs)
│   ├── auth.ts        # login(), verify2fa(), me()
│   ├── tasks.ts       # createTask(), listTasks(), assignTasks(), viewMyTasks(), updateTask()
│   ├── users.ts       # listStaff()
│   └── dev.ts         # latestEmail(email)
├── auth/
│   ├── AuthContext.tsx   # { token, user, setSession, logout }, persisted in sessionStorage
│   └── ProtectedRoute.tsx
├── hooks/
│   └── useAsync.ts    # { data, error, loading, run } — one pattern for every call
├── components/
│   ├── StatusMessage.tsx  # <Loading/>, <ErrorMessage error/>, <Empty text/>, <Success text/>
│   ├── TaskList.tsx       # pure presentational list/table
│   ├── TaskForm.tsx       # controlled form, onSubmit prop
│   ├── AssignPanel.tsx    # staff select + selected count + button
│   ├── CacheBadge.tsx
│   └── NavBar.tsx         # user email/role + logout
├── pages/
│   ├── LoginPage.tsx
│   ├── AdminPage.tsx
│   └── MyTasksPage.tsx
├── App.tsx            # routes
└── main.tsx
```

**Rule:** components never call `fetch`. Pages call functions in `api/` through `useAsync`. Presentational components only receive props.

## API client

```ts
export class ApiError extends Error {
  constructor(public status: number, public code: string, message: string) { super(message); }
}

export async function request<T>(method: string, path: string, body?: unknown): Promise<T> {
  const token = getToken();                       // from AuthContext storage
  const res = await fetch(`${BASE_URL}${path}`, {
    method,
    headers: { 'Content-Type': 'application/json', ...(token && { Authorization: `Bearer ${token}` }) },
    body: body ? JSON.stringify(body) : undefined,
  });
  if (res.status === 401 && token) onUnauthorized();  // expired token → logout → /login
  const data = await res.json().catch(() => null);
  if (!res.ok) throw new ApiError(res.status, data?.error?.code ?? 'unknown', data?.error?.message ?? res.statusText);
  return data as T;
}
```

`ErrorMessage` maps known codes to friendly text: `forbidden`, `invalid_code`, `code_expired`, `code_already_used`, `too_many_attempts`, plus a network-failure message ("Cannot reach API at …").

## UI states (required by the brief)

| State | Where it appears |
|---|---|
| **Loading** | The login and verify buttons are disabled and show "Verifying…". The task lists show "Loading tasks…" |
| **Error** | Bad credentials, wrong or expired code, the 403 on create, the API being unreachable |
| **Empty** | "No tasks assigned to you yet." / "No tasks created yet." |
| **Success** | "Task created", "3 tasks assigned to jamesbond@example.com" |

## Token storage trade-off

The token lives in `sessionStorage`, which is cleared when the tab closes. Unlike an `httpOnly` cookie, it is readable by JavaScript, so it would be exposed by an XSS bug. This was chosen because it is simple to explain and the brief asks to *"use the JWT returned by the Rust API"* in requests. The production alternative is documented: an `httpOnly` SameSite cookie with a CSRF token.

## Styling

Plain CSS in one `index.css`: a centered container, a simple table, and a responsive layout down to phone width. No component library.
