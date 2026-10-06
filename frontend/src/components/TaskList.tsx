import type { Task } from '../api/types'
import { Empty } from './StatusMessage'

interface Props {
  tasks: Task[]
  emptyText: string
  /** When provided, each row gets a checkbox. */
  selectedIds?: Set<string>
  onToggle?: (id: string) => void
}

const STATUS_LABEL: Record<Task['status'], string> = {
  todo: 'To do',
  in_progress: 'In progress',
  done: 'Done',
}

export default function TaskList({ tasks, emptyText, selectedIds, onToggle }: Props) {
  if (tasks.length === 0) return <Empty text={emptyText} />
  const selectable = Boolean(selectedIds && onToggle)

  return (
    <div className="table-wrap">
      <table>
        <thead>
          <tr>
            {selectable && <th aria-label="Select" />}
            <th>Title</th>
            <th>Priority</th>
            <th>Status</th>
            <th>Assigned to</th>
          </tr>
        </thead>
        <tbody>
          {tasks.map((task) => (
            <tr key={task.id}>
              {selectable && (
                <td>
                  <input
                    type="checkbox"
                    aria-label={`Select ${task.title}`}
                    checked={selectedIds!.has(task.id)}
                    onChange={() => onToggle!(task.id)}
                  />
                </td>
              )}
              <td>{task.title}</td>
              <td>
                <span className={`badge priority-${task.priority}`}>{task.priority}</span>
              </td>
              <td>{STATUS_LABEL[task.status]}</td>
              <td>{task.assigned_to ?? '—'}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  )
}
