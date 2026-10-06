import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { MemoryRouter } from 'react-router-dom'
import { describe, expect, it } from 'vitest'
import { AppRoutes } from './App'
import { AuthProvider } from './auth/AuthContext'
import { loadSession, saveSession } from './auth/session'
import { admin, james, mockApi, myTasksBody } from './test/mockApi'

function renderApp(path: string) {
  return render(
    <AuthProvider>
      <MemoryRouter initialEntries={[path]}>
        <AppRoutes />
      </MemoryRouter>
    </AuthProvider>,
  )
}

describe('App routing', () => {
  it('sends anonymous users to the login screen', async () => {
    renderApp('/my-tasks')

    expect(await screen.findByRole('heading', { name: 'Sign in' })).toBeInTheDocument()
  })

  it('keeps staff out of the admin screen', async () => {
    saveSession({ token: 'jwt-james', user: james })
    mockApi({ 'GET /tasks/view-my-tasks': { status: 200, body: myTasksBody(false) } })

    renderApp('/admin')

    expect(await screen.findByRole('heading', { name: 'My tasks' })).toBeInTheDocument()
  })

  it('lands admins on the admin screen', async () => {
    saveSession({ token: 'jwt-admin', user: admin })
    mockApi({ 'GET /tasks': { status: 200, body: [] }, 'GET /users': { status: 200, body: [james] } })

    renderApp('/')

    expect(await screen.findByRole('heading', { name: 'Create task' })).toBeInTheDocument()
  })

  it('logs out on an expired token (401) and returns to login', async () => {
    saveSession({ token: 'expired', user: james })
    mockApi({
      'GET /tasks/view-my-tasks': { status: 401, body: { error: { code: 'unauthorized', message: 'Invalid or expired token' } } },
    })

    renderApp('/my-tasks')

    expect(await screen.findByRole('heading', { name: 'Sign in' })).toBeInTheDocument()
    expect(loadSession()).toBeNull()
  })

  it('log out button clears the session', async () => {
    saveSession({ token: 'jwt-james', user: james })
    mockApi({ 'GET /tasks/view-my-tasks': { status: 200, body: myTasksBody(false) } })
    renderApp('/my-tasks')

    await userEvent.setup().click(await screen.findByRole('button', { name: 'Log out' }))

    expect(await screen.findByRole('heading', { name: 'Sign in' })).toBeInTheDocument()
    expect(loadSession()).toBeNull()
  })
})
