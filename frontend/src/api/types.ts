// Mirrors the Rust API DTOs (backend/src/dto).

export type Role = 'admin' | 'staff'
export type TaskStatus = 'todo' | 'in_progress' | 'done'
export type TaskPriority = 'low' | 'medium' | 'high'

export interface User {
  id: string
  full_name: string
  email: string
  role: Role
}

export interface Task {
  id: string
  title: string
  description: string
  status: TaskStatus
  priority: TaskPriority
  assigned_to: string | null
  created_by: string
  created_at: string
  updated_at: string
}

export interface LoginResponse {
  login_challenge_id: string
  expires_in_seconds: number
  message: string
}

export interface TokenResponse {
  access_token: string
  token_type: 'Bearer'
  expires_in_seconds: number
  user: User
}

export interface CreateTaskInput {
  title: string
  description?: string
  priority?: TaskPriority
}

export interface AssignTasksResponse {
  assigned_to: string
  task_ids: string[]
  updated_count: number
}

export interface MyTasksResponse {
  user: { email: string; role: Role }
  tasks: Task[]
  summary: { total_assigned_tasks: number }
  cache: { hit: boolean }
}

export interface DevEmail {
  id: string
  to_email: string
  subject: string
  body: string
  code: string | null
  challenge_id: string | null
  created_at: string
}
