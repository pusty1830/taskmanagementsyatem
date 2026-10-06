import { useState, type FormEvent } from 'react'
import type { CreateTaskInput, TaskPriority } from '../api/types'

interface Props {
  /** Resolve true to clear the form (success). */
  onSubmit: (input: CreateTaskInput) => Promise<boolean>
  submitting: boolean
}

export default function TaskForm({ onSubmit, submitting }: Props) {
  const [title, setTitle] = useState('')
  const [description, setDescription] = useState('')
  const [priority, setPriority] = useState<TaskPriority>('medium')

  async function handleSubmit(e: FormEvent) {
    e.preventDefault()
    const ok = await onSubmit({ title, description, priority })
    if (ok) {
      setTitle('')
      setDescription('')
      setPriority('medium')
    }
  }

  return (
    <form className="form" onSubmit={handleSubmit}>
      <label>
        Title
        <input value={title} onChange={(e) => setTitle(e.target.value)} required maxLength={200} />
      </label>
      <label>
        Description
        <textarea value={description} onChange={(e) => setDescription(e.target.value)} maxLength={2000} rows={2} />
      </label>
      <label>
        Priority
        <select value={priority} onChange={(e) => setPriority(e.target.value as TaskPriority)}>
          <option value="low">low</option>
          <option value="medium">medium</option>
          <option value="high">high</option>
        </select>
      </label>
      <button type="submit" disabled={submitting}>
        {submitting ? 'Creating…' : 'Create task'}
      </button>
    </form>
  )
}
