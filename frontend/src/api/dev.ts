import { request } from './client'
import type { DevEmail } from './types'

/** Development mailbox: latest verification email for `email`. */
export const latestEmail = (email: string) =>
  request<DevEmail>('GET', `/dev/email-logs/latest?email=${encodeURIComponent(email)}`)
