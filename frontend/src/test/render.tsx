import { render } from '@testing-library/react'
import type { ReactElement } from 'react'
import { MemoryRouter, Route, Routes } from 'react-router-dom'
import { AuthProvider } from '../auth/AuthContext'
import { saveSession, type Session } from '../auth/session'

/** Renders `ui` at `path` with auth + router, plus stub screens to observe redirects. */
export function renderAt(path: string, ui: ReactElement, session?: Session) {
  if (session) saveSession(session)
  return render(
    <AuthProvider>
      <MemoryRouter initialEntries={[path]}>
        <Routes>
          <Route path={path} element={ui} />
          {path !== '/admin' && <Route path="/admin" element={<p>Admin screen</p>} />}
          {path !== '/my-tasks' && <Route path="/my-tasks" element={<p>My tasks screen</p>} />}
          {path !== '/login' && <Route path="/login" element={<p>Login screen</p>} />}
        </Routes>
      </MemoryRouter>
    </AuthProvider>,
  )
}
