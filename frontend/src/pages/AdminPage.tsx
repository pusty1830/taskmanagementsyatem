import { useCallback, useEffect, useRef, useState } from 'react'
import * as tasksApi from '../api/tasks'
import * as usersApi from '../api/users'
import type { CreateTaskInput } from '../api/types'
import AssignPanel from '../components/AssignPanel'
import { ErrorMessage, Loading, Success } from '../components/StatusMessage'
import TaskForm from '../components/TaskForm'
import TaskList from '../components/TaskList'
import { useAsync } from '../hooks/useAsync'

export default function AdminPage() {
  const tasks = useAsync(tasksApi.listTasks, true)
  const staff = useAsync(usersApi.listStaff, true)
  const create = useAsync(tasksApi.createTask)
  const assign = useAsync(tasksApi.assignTasks)

  const [selected, setSelected] = useState<Set<string>>(new Set())
  const [success, setSuccess] = useState<string | null>(null)

  const { run: loadTasks } = tasks
  const { run: loadStaff } = staff
  const loaded = useRef(false)
  useEffect(() => {
    if (loaded.current) return
    loaded.current = true
    void loadTasks()
    void loadStaff()
  }, [loadTasks, loadStaff])

  const toggle = useCallback((id: string) => {
    setSelected((prev) => {
      const next = new Set(prev)
      if (next.has(id)) next.delete(id)
      else next.add(id)
      return next
    })
  }, [])

  async function handleCreate(input: CreateTaskInput) {
    setSuccess(null)
    const task = await create.run(input)
    if (!task) return false
    setSuccess(`Task created: ${task.title}`)
    await loadTasks()
    return true
  }

  async function handleAssign(email: string) {
    setSuccess(null)
    // Keep the on-screen order, so the request lists ids as the admin sees them.
    const ids = (tasks.data ?? []).filter((t) => selected.has(t.id)).map((t) => t.id)
    const res = await assign.run(ids, email)
    if (!res) return
    setSuccess(`${res.updated_count} tasks assigned to ${res.assigned_to}`)
    setSelected(new Set())
    await loadTasks()
  }

  return (
    <main className="stack">
      <section className="card">
        <h1>Create task</h1>
        <TaskForm onSubmit={handleCreate} submitting={create.loading} />
        <ErrorMessage error={create.error} />
      </section>

      <section className="card">
        <div className="section-head">
          <h2>All tasks {tasks.data && `(${tasks.data.length})`}</h2>
          <AssignPanel
            staff={staff.data ?? []}
            selectedCount={selected.size}
            assigning={assign.loading}
            onAssign={handleAssign}
          />
        </div>
        {success && <Success text={success} />}
        <ErrorMessage error={assign.error ?? staff.error ?? tasks.error} />
        {tasks.loading && !tasks.data && <Loading text="Loading tasks…" />}
        {tasks.data && (
          <TaskList tasks={tasks.data} emptyText="No tasks created yet." selectedIds={selected} onToggle={toggle} />
        )}
      </section>
    </main>
  )
}
