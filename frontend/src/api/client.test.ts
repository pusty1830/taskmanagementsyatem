import { describe, expect, it, vi } from 'vitest'
import { ApiError, request } from './client'
import { saveSession } from '../auth/session'
import { james, mockApi } from '../test/mockApi'

describe('api client', () => {
  it('attaches the JWT as a Bearer header when a session exists', async () => {
    saveSession({ token: 'jwt-123', user: james })
    const fetchMock = mockApi({ 'GET /auth/me': { status: 200, body: james } })

    await request('GET', '/auth/me')

    const init = fetchMock.mock.calls[0][1] as RequestInit
    expect((init.headers as Record<string, string>).Authorization).toBe('Bearer jwt-123')
  })

  it('sends no Authorization header without a session', async () => {
    const fetchMock = mockApi({ 'POST /auth/login': { status: 200, body: {} } })

    await request('POST', '/auth/login', { email: 'a@b.c', password: 'x' })

    const init = fetchMock.mock.calls[0][1] as RequestInit
    expect((init.headers as Record<string, string>).Authorization).toBeUndefined()
    expect(init.body).toBe(JSON.stringify({ email: 'a@b.c', password: 'x' }))
  })

  it('maps the API error envelope to ApiError', async () => {
    mockApi({
      'POST /tasks': {
        status: 403,
        body: { error: { code: 'forbidden', message: 'Only admins can create tasks' } },
      },
    })

    const err = await request('POST', '/tasks', { title: 'x' }).catch((e: unknown) => e)

    expect(err).toBeInstanceOf(ApiError)
    expect(err).toMatchObject({ status: 403, code: 'forbidden', message: 'Only admins can create tasks' })
  })

  it('reports an unreachable API as a network error', async () => {
    vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new TypeError('Failed to fetch')))

    const err = await request('GET', '/health').catch((e: unknown) => e)

    expect(err).toMatchObject({ status: 0, code: 'network_error' })
    // A CORS rejection looks identical to a down server, so the message names the page origin too.
    expect((err as Error).message).toContain(`CORS_ORIGIN allows ${window.location.origin}`)
  })
})
