import { request } from './client'
import type { User } from './types'

export const listStaff = () => request<User[]>('GET', '/users?role=staff')
