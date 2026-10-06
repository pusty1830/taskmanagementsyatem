import { screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it } from 'vitest'
import { loadSession } from '../auth/session'
import { james, mockApi } from '../test/mockApi'
import { renderAt } from '../test/render'
import LoginPage from './LoginPage'

const loginOk = {
  status: 200,
  body: { login_challenge_id: 'ch-1', expires_in_seconds: 300, message: 'Verification code sent to j***d@example.com' },
}

async function submitCredentials() {
  const user = userEvent.setup()
  await user.type(screen.getByLabelText('Email'), james.email)
  await user.type(screen.getByLabelText('Password'), 'JamesBond@007')
  await user.click(screen.getByRole('button', { name: 'Continue' }))
  return user
}

describe('LoginPage', () => {
  it('moves to the 2FA step after valid credentials, without storing a token', async () => {
    mockApi({ 'POST /auth/login': loginOk })
    renderAt('/login', <LoginPage />)

    await submitCredentials()

    expect(await screen.findByText('Verification code sent to j***d@example.com')).toBeInTheDocument()
    expect(screen.getByLabelText('Verification code')).toBeInTheDocument()
    expect(loadSession()).toBeNull()
  })

  it('shows an error for bad credentials', async () => {
    mockApi({
      'POST /auth/login': { status: 401, body: { error: { code: 'unauthorized', message: 'Invalid email or password' } } },
    })
    renderAt('/login', <LoginPage />)

    await submitCredentials()

    expect(await screen.findByRole('alert')).toHaveTextContent('Invalid email or password')
  })

  it('rejects a wrong code with a clear message', async () => {
    mockApi({
      'POST /auth/login': loginOk,
      'POST /auth/verify-2fa': { status: 401, body: { error: { code: 'invalid_code', message: 'Invalid verification code' } } },
    })
    renderAt('/login', <LoginPage />)
    const user = await submitCredentials()

    await user.type(await screen.findByLabelText('Verification code'), '000000')
    await user.click(screen.getByRole('button', { name: 'Verify' }))

    expect(await screen.findByRole('alert')).toHaveTextContent('Incorrect verification code')
  })

  it('fills the code from the dev mailbox, verifies and redirects staff to their tasks', async () => {
    const fetchMock = mockApi({
      'POST /auth/login': loginOk,
      'GET /dev/email-logs/latest': { status: 200, body: { code: '482913' } },
      'POST /auth/verify-2fa': {
        status: 200,
        body: { access_token: 'jwt-james', token_type: 'Bearer', expires_in_seconds: 3600, user: james },
      },
    })
    renderAt('/login', <LoginPage />)
    const user = await submitCredentials()

    await user.click(await screen.findByRole('button', { name: /dev mailbox/i }))
    expect(screen.getByLabelText('Verification code')).toHaveValue('482913')
    await user.click(screen.getByRole('button', { name: 'Verify' }))

    expect(await screen.findByText('My tasks screen')).toBeInTheDocument()
    expect(loadSession()?.token).toBe('jwt-james')
    const verifyCall = fetchMock.mock.calls.find(([url]) => String(url).endsWith('/auth/verify-2fa'))!
    expect(JSON.parse(String(verifyCall[1]!.body))).toEqual({ login_challenge_id: 'ch-1', code: '482913' })
  })
})
