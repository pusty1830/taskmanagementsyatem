import { vi } from 'vitest'

type Reply = { status: number; body?: unknown }
type Handler = Reply | Reply[] | ((init: RequestInit) => Reply)

/**
 * Replaces global fetch with a router keyed by "METHOD /path".
 * An array handler replies in order (last reply repeats), so a test can script MISS then HIT.
 */
export function mockApi(routes: Record<string, Handler>) {
  const counters = new Map<string, number>()
  const fetchMock = vi.fn(async (input: RequestInfo | URL, init: RequestInit = {}) => {
    const url = new URL(String(input))
    const key = `${init.method ?? 'GET'} ${url.pathname}`
    const handler = routes[key]
    if (!handler) throw new Error(`Unmocked request: ${key}`)

    let reply: Reply
    if (typeof handler === 'function') reply = handler(init)
    else if (Array.isArray(handler)) {
      const n = counters.get(key) ?? 0
      counters.set(key, n + 1)
      reply = handler[Math.min(n, handler.length - 1)]
    } else reply = handler

    return new Response(reply.body === undefined ? null : JSON.stringify(reply.body), {
      status: reply.status,
      headers: { 'Content-Type': 'application/json' },
    })
  })
  vi.stubGlobal('fetch', fetchMock)
  return fetchMock
}

export const sampleTasks = [
  { id: 't1', title: 'Infiltrate SPECTRE HQ', description: '', status: 'todo', priority: 'high', assigned_to: 'jamesbond@example.com', created_by: 'admin@example.com', created_at: '2026-10-06T10:00:00Z', updated_at: '2026-10-06T10:00:00Z' },
  { id: 't2', title: 'Recover the Lektor', description: '', status: 'todo', priority: 'medium', assigned_to: 'jamesbond@example.com', created_by: 'admin@example.com', created_at: '2026-10-06T10:00:00Z', updated_at: '2026-10-06T10:00:00Z' },
  { id: 't3', title: 'Brief M on Blofeld', description: '', status: 'todo', priority: 'low', assigned_to: 'jamesbond@example.com', created_by: 'admin@example.com', created_at: '2026-10-06T10:00:00Z', updated_at: '2026-10-06T10:00:00Z' },
]

export const james = { id: 'u2', full_name: 'James Bond', email: 'jamesbond@example.com', role: 'staff' as const }
export const admin = { id: 'u1', full_name: 'Admin', email: 'admin@example.com', role: 'admin' as const }

export function myTasksBody(hit: boolean, tasks = sampleTasks) {
  return {
    user: { email: james.email, role: 'staff' },
    tasks,
    summary: { total_assigned_tasks: tasks.length },
    cache: { hit },
  }
}
