import { useState, type FormEvent } from 'react'
import { Navigate, useNavigate } from 'react-router-dom'
import * as authApi from '../api/auth'
import * as devApi from '../api/dev'
import { useAuth } from '../auth/AuthContext'
import { ErrorMessage } from '../components/StatusMessage'
import { useAsync } from '../hooks/useAsync'

const homeFor = (role: string) => (role === 'admin' ? '/admin' : '/my-tasks')

export default function LoginPage() {
  const { session, signIn } = useAuth()
  const navigate = useNavigate()

  const [email, setEmail] = useState('')
  const [password, setPassword] = useState('')
  const [code, setCode] = useState('')
  const [challenge, setChallenge] = useState<{ id: string; message: string } | null>(null)

  const login = useAsync(authApi.login)
  const verify = useAsync(authApi.verify2fa)
  const mailbox = useAsync(devApi.latestEmail)

  if (session) return <Navigate to={homeFor(session.user.role)} replace />

  async function submitCredentials(e: FormEvent) {
    e.preventDefault()
    const res = await login.run(email, password)
    if (res) setChallenge({ id: res.login_challenge_id, message: res.message })
  }

  async function submitCode(e: FormEvent) {
    e.preventDefault()
    if (!challenge) return
    const res = await verify.run(challenge.id, code.trim())
    if (res) {
      signIn(res.access_token, res.user)
      navigate(homeFor(res.user.role), { replace: true })
    }
  }

  async function fillFromMailbox() {
    const mail = await mailbox.run(email)
    if (mail?.code) setCode(mail.code)
  }

  function startOver() {
    setChallenge(null)
    setCode('')
    verify.reset()
    mailbox.reset()
  }

  return (
    <main className="card narrow">
      <h1>Sign in</h1>

      {!challenge ? (
        <form className="form" onSubmit={submitCredentials}>
          <label>
            Email
            <input type="email" value={email} onChange={(e) => setEmail(e.target.value)} required autoComplete="username" />
          </label>
          <label>
            Password
            <input
              type="password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              required
              autoComplete="current-password"
            />
          </label>
          <button type="submit" disabled={login.loading}>
            {login.loading ? 'Signing in…' : 'Continue'}
          </button>
          <ErrorMessage error={login.error} />
          <p className="hint">
            Seeded users: <code>admin@example.com / Admin@12345</code>, <code>jamesbond@example.com / JamesBond@007</code>
          </p>
        </form>
      ) : (
        <form className="form" onSubmit={submitCode}>
          <h2>Enter verification code</h2>
          <p>{challenge.message}</p>
          <label>
            Verification code
            <input
              value={code}
              onChange={(e) => setCode(e.target.value)}
              inputMode="numeric"
              pattern="\d{6}"
              maxLength={6}
              required
              autoComplete="one-time-code"
            />
          </label>
          <button type="submit" disabled={verify.loading}>
            {verify.loading ? 'Verifying…' : 'Verify'}
          </button>
          <ErrorMessage error={verify.error ?? mailbox.error} />
          <div className="row">
            <button type="button" className="secondary" onClick={fillFromMailbox} disabled={mailbox.loading}>
              {mailbox.loading ? 'Checking mailbox…' : 'Fetch code from dev mailbox'}
            </button>
            <button type="button" className="link" onClick={startOver}>
              Start over
            </button>
          </div>
        </form>
      )}
    </main>
  )
}
