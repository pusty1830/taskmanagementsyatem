import { request } from './client'
import type { LoginResponse, TokenResponse, User } from './types'

export const login = (email: string, password: string) =>
  request<LoginResponse>('POST', '/auth/login', { email, password })

export const verify2fa = (loginChallengeId: string, code: string) =>
  request<TokenResponse>('POST', '/auth/verify-2fa', { login_challenge_id: loginChallengeId, code })

export const me = () => request<User>('GET', '/auth/me')
