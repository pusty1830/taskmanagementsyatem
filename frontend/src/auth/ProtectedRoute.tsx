import type { ReactNode } from 'react'
import { Navigate } from 'react-router-dom'
import type { Role } from '../api/types'
import { useAuth } from './AuthContext'

/** UX guard only: the Rust API enforces every permission itself. */
export default function ProtectedRoute({ role, children }: { role?: Role; children: ReactNode }) {
  const { session } = useAuth()
  if (!session) return <Navigate to="/login" replace />
  if (role && session.user.role !== role) return <Navigate to="/my-tasks" replace />
  return <>{children}</>
}
