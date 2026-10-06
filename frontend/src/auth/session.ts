import type { User } from '../api/types'

export interface Session {
  token: string
  user: User
}

// sessionStorage: survives reloads, cleared when the tab closes (trade-off documented in docs/07).
const KEY = 'taskapp.session'

export function loadSession(): Session | null {
  try {
    const raw = sessionStorage.getItem(KEY)
    return raw ? (JSON.parse(raw) as Session) : null
  } catch {
    return null
  }
}

export function saveSession(session: Session): void {
  try {
    sessionStorage.setItem(KEY, JSON.stringify(session))
  } catch {
    // Storage unavailable (private mode): the session lives in memory only.
  }
}

export function clearSession(): void {
  try {
    sessionStorage.removeItem(KEY)
  } catch {
    // ignore
  }
}
