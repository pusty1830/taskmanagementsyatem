import { useEffect, useRef, useState } from 'react'
import * as tasksApi from '../api/tasks'
import type { CreateTaskInput } from '../api/types'
import { useAuth } from '../auth/AuthContext'
import CacheBadge from '../components/CacheBadge'
import { ErrorMessage, Loading, Success } from '../components/StatusMessage'
import TaskForm from '../components/TaskForm'
import TaskList from '../components/TaskList'
import { useAsync } from '../hooks/useAsync'

export default function MyTasksPage() {
  const { session } = useAuth()
  const myTasks = useAsync(tasksApi.viewMyTasks, true)
  const create = useAsync(tasksApi.createTask)
  const [created, setCreated] = useState<string | null>(null)

  // Fetch once on mount. The ref stops React StrictMode's double effect from making a second
  // request, which would otherwise turn the first visible response into a cache HIT.
  const loaded = useRef(false)
  const { run: loadMyTasks } = myTasks
  useEffect(() => {
    if (loaded.current) return
    loaded.current = true
    void loadMyTasks()
  }, [loadMyTasks])

  async function tryCreate(input: CreateTaskInput) {
    setCreated(null)
    const task = await create.run(input)
    if (task) setCreated(task.title)
    return Boolean(task)
  }

  const data = myTasks.data
  const isStaff = session?.user.role === 'staff'

  return (
    <main className="stack">
      <section className="card">
        <div className="section-head">
          <h1>My tasks</h1>
          <div className="row">
            {data && <span>{data.summary.total_assigned_tasks} assigned tasks</span>}
            {data && <CacheBadge hit={data.cache.hit} />}
            <button type="button" className="secondary" onClick={() => void loadMyTasks()} disabled={myTasks.loading}>
              Refresh
            </button>
          </div>
        </div>
        {myTasks.loading && !data && <Loading text="Loading tasks…" />}
        <ErrorMessage error={myTasks.error} />
        {data && <TaskList tasks={data.tasks} emptyText="No tasks assigned to you yet." />}
      </section>

      {isStaff && (
        <section className="card">
          <h2>Try creating a task</h2>
          <p className="hint">Only admins can create tasks — the API answers staff with 403 Forbidden.</p>
          <TaskForm onSubmit={tryCreate} submitting={create.loading} />
          <ErrorMessage error={create.error} />
          {created && <Success text={`Task created: ${created}`} />}
        </section>
      )}
    </main>
  )
}
