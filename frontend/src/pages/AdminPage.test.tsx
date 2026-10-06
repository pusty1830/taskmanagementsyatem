import { screen, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it } from 'vitest'
import { admin, james, mockApi, sampleTasks } from '../test/mockApi'
import { renderAt } from '../test/render'
import AdminPage from './AdminPage'

const session = { token: 'jwt-admin', user: admin }
const unassigned = sampleTasks.map((t) => ({ ...t, assigned_to: null }))
const fiveTasks = [
  ...unassigned,
  { ...unassigned[0], id: 't4', title: 'Service the Aston Martin' },
  { ...unassigned[0], id: 't5', title: 'Collect gadgets from Q' },
]

describe('AdminPage', () => {
  it('shows the empty state when no tasks exist', async () => {
    mockApi({
      'GET /tasks': { status: 200, body: [] },
      'GET /users': { status: 200, body: [james] },
    })
    renderAt('/admin', <AdminPage />, session)

    expect(await screen.findByText('No tasks created yet.')).toBeInTheDocument()
  })

  it('creates a task and refreshes the list', async () => {
    const fetchMock = mockApi({
      'GET /tasks': [
        { status: 200, body: [] },
        { status: 200, body: [unassigned[0]] },
      ],
      'GET /users': { status: 200, body: [james] },
      'POST /tasks': { status: 201, body: unassigned[0] },
    })
    renderAt('/admin', <AdminPage />, session)
    const user = userEvent.setup()
    await screen.findByText('No tasks created yet.')

    await user.type(screen.getByLabelText('Title'), 'Infiltrate SPECTRE HQ')
    await user.selectOptions(screen.getByLabelText('Priority'), 'high')
    await user.click(screen.getByRole('button', { name: 'Create task' }))

    expect(await screen.findByText('Task created: Infiltrate SPECTRE HQ')).toBeInTheDocument()
    expect(await screen.findByRole('cell', { name: 'Infiltrate SPECTRE HQ' })).toBeInTheDocument()
    const createCall = fetchMock.mock.calls.find(([, init]) => init?.method === 'POST')!
    expect(JSON.parse(String(createCall[1]!.body))).toMatchObject({ title: 'Infiltrate SPECTRE HQ', priority: 'high' })
  })

  it('assigns exactly the selected tasks to the chosen staff member', async () => {
    const fetchMock = mockApi({
      'GET /tasks': { status: 200, body: fiveTasks },
      'GET /users': { status: 200, body: [james] },
      'POST /tasks/assign': {
        status: 200,
        body: { assigned_to: james.email, task_ids: ['t1', 't2', 't3'], updated_count: 3 },
      },
    })
    renderAt('/admin', <AdminPage />, session)
    const user = userEvent.setup()
    await screen.findByText('Collect gadgets from Q')

    for (const title of ['Infiltrate SPECTRE HQ', 'Recover the Lektor', 'Brief M on Blofeld']) {
      const row = screen.getByRole('row', { name: new RegExp(title) })
      await user.click(within(row).getByRole('checkbox'))
    }
    await user.click(screen.getByRole('button', { name: 'Assign selected (3)' }))

    expect(await screen.findByText('3 tasks assigned to jamesbond@example.com')).toBeInTheDocument()
    const assignCall = fetchMock.mock.calls.find(([url]) => String(url).endsWith('/tasks/assign'))!
    expect(JSON.parse(String(assignCall[1]!.body))).toEqual({
      task_ids: ['t1', 't2', 't3'],
      assignee_email: james.email,
    })
  })
})
