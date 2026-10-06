import { NavLink } from 'react-router-dom'
import { useAuth } from '../auth/AuthContext'

export default function NavBar() {
  const { session, signOut } = useAuth()
  if (!session) return null
  const { user } = session

  return (
    <header className="navbar">
      <strong>Task Manager</strong>
      <nav>
        {user.role === 'admin' && <NavLink to="/admin">Admin</NavLink>}
        <NavLink to="/my-tasks">My tasks</NavLink>
      </nav>
      <span className="who">
        {user.email} <span className="badge">{user.role}</span>
      </span>
      <button type="button" className="link" onClick={signOut}>
        Log out
      </button>
    </header>
  )
}
