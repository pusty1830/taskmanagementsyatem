import { request } from './client'
import type { AssignTasksResponse, CreateTaskInput, MyTasksResponse, Task, TaskStatus } from './types'

export const createTask = (input: CreateTaskInput) => request<Task>('POST', '/tasks', input)

export const listTasks = () => request<Task[]>('GET', '/tasks')

export const assignTasks = (taskIds: string[], assigneeEmail: string) =>
  request<AssignTasksResponse>('POST', '/tasks/assign', { task_ids: taskIds, assignee_email: assigneeEmail })

export const updateTaskStatus = (id: string, status: TaskStatus) =>
  request<Task>('PATCH', `/tasks/${id}`, { status })

export const viewMyTasks = () => request<MyTasksResponse>('GET', '/tasks/view-my-tasks')
