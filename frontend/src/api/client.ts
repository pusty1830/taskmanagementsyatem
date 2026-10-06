import { loadSession } from '../auth/session'

export const API_BASE_URL: string = import.meta.env.VITE_API_BASE_URL ?? 'http://localhost:8080'

/** Error raised for any non-2xx response, carrying the API's `error.code`. */
export class ApiError extends Error {
  readonly status: number
  readonly code: string
  readonly details?: unknown

  constructor(status: number, code: string, message: string, details?: unknown) {
    super(message)
    this.name = 'ApiError'
    this.status = status
    this.code = code
    this.details = details
  }
}

let onUnauthorized: (() => void) | null = null

/** Called when an authenticated request gets 401 (expired/invalid token). */
export function setUnauthorizedHandler(handler: (() => void) | null): void {
  onUnauthorized = handler
}

export async function request<T>(method: string, path: string, body?: unknown): Promise<T> {
  const token = loadSession()?.token
  const headers: Record<string, string> = { 'Content-Type': 'application/json' }
  if (token) headers.Authorization = `Bearer ${token}`

  let res: Response
  try {
    res = await fetch(`${API_BASE_URL}${path}`, {
      method,
      headers,
      body: body === undefined ? undefined : JSON.stringify(body),
    })
  } catch {
    throw new ApiError(0, 'network_error', `Cannot reach the API at ${API_BASE_URL}. Is the backend running?`)
  }

  const data: unknown = await res.json().catch(() => null)
  if (!res.ok) {
    if (res.status === 401 && token) onUnauthorized?.()
    const err = (data as { error?: { code?: string; message?: string; details?: unknown } } | null)?.error
    throw new ApiError(res.status, err?.code ?? 'unknown', err?.message ?? res.statusText, err?.details)
  }
  return data as T
}
