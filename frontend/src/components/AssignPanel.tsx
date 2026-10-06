import { useState } from 'react'
import type { User } from '../api/types'

interface Props {
  staff: User[]
  selectedCount: number
  assigning: boolean
  onAssign: (assigneeEmail: string) => void
}

export default function AssignPanel({ staff, selectedCount, assigning, onAssign }: Props) {
  const [chosen, setChosen] = useState('')
  const assignee = chosen || staff[0]?.email || ''

  return (
    <div className="assign-panel">
      <label>
        Assign to
        <select value={assignee} onChange={(e) => setChosen(e.target.value)} disabled={staff.length === 0}>
          {staff.map((u) => (
            <option key={u.id} value={u.email}>
              {u.full_name} ({u.email})
            </option>
          ))}
        </select>
      </label>
      <button
        type="button"
        disabled={assigning || selectedCount === 0 || !assignee}
        onClick={() => onAssign(assignee)}
      >
        {assigning ? 'Assigning…' : `Assign selected (${selectedCount})`}
      </button>
    </div>
  )
}
