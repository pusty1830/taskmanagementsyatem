import { screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import { james, mockApi, myTasksBody } from '../test/mockApi'
import { renderAt } from '../test/render'
import MyTasksPage from './MyTasksPage'

const session = { token: 'jwt-james', user: james }

describe('MyTasksPage', () => {
  it('shows loading, then the 3 API tasks with a cache MISS, then HIT after refresh', async () => {
    mockApi({
      'GET /tasks/view-my-tasks': [
        { status: 200, body: myTasksBody(false) },
        { status: 200, body: myTasksBody(true) },
      ],
    })
    renderAt('/my-tasks', <MyTasksPage />, session)

    expect(screen.getByText('Loading tasks…')).toBeInTheDocument()
    expect(await screen.findByText('Infiltrate SPECTRE HQ')).toBeInTheDocument()
    expect(screen.getByText('Recover the Lektor')).toBeInTheDocument()
    expect(screen.getByText('Brief M on Blofeld')).toBeInTheDocument()
    expect(screen.getByText('3 assigned tasks')).toBeInTheDocument()
    expect(screen.getByText('cache: MISS')).toBeInTheDocument()

    await userEvent.setup().click(screen.getByRole('button', { name: 'Refresh' }))

    expect(await screen.findByText('cache: HIT')).toBeInTheDocument()
  })

  it('loads once on mount', async () => {
    const fetchMock = mockApi({ 'GET /tasks/view-my-tasks': { status: 200, body: myTasksBody(false) } })
    renderAt('/my-tasks', <MyTasksPage />, session)

    await screen.findByText('cache: MISS')

    expect(fetchMock).toHaveBeenCalledTimes(1)
  })

  it('shows the empty state', async () => {
    mockApi({ 'GET /tasks/view-my-tasks': { status: 200, body: myTasksBody(false, []) } })
    renderAt('/my-tasks', <MyTasksPage />, session)

    expect(await screen.findByText('No tasks assigned to you yet.')).toBeInTheDocument()
  })

  it('shows an error when the API is unreachable', async () => {
    vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new TypeError('Failed to fetch')))
    renderAt('/my-tasks', <MyTasksPage />, session)

    expect(await screen.findByRole('alert')).toHaveTextContent('Cannot reach the API')
  })

  it('shows a clear message when staff try to create a task (403)', async () => {
    mockApi({
      'GET /tasks/view-my-tasks': { status: 200, body: myTasksBody(false) },
      'POST /tasks': { status: 403, body: { error: { code: 'forbidden', message: 'Only admins can create tasks' } } },
    })
    renderAt('/my-tasks', <MyTasksPage />, session)
    const user = userEvent.setup()
    await screen.findByText('cache: MISS')

    await user.type(screen.getByLabelText('Title'), 'License to create')
    await user.click(screen.getByRole('button', { name: 'Create task' }))

    expect(await screen.findByRole('alert')).toHaveTextContent('Forbidden: Only admins can create tasks (403)')
  })
})
