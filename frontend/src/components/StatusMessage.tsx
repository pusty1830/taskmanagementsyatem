import { ApiError } from '../api/client'

export function Loading({ text = 'Loading…' }: { text?: string }) {
  return (
    <p className="status status-loading" role="status">
      {text}
    </p>
  )
}

export function Empty({ text }: { text: string }) {
  return <p className="status status-empty">{text}</p>
}

export function Success({ text }: { text: string }) {
  return (
    <p className="status status-success" role="status">
      {text}
    </p>
  )
}

const FRIENDLY: Record<string, string> = {
  invalid_code: 'Incorrect verification code. Please try again.',
  code_expired: 'This code has expired. Please start the login again.',
  code_already_used: 'This code was already used. Please start the login again.',
  too_many_attempts: 'Too many incorrect attempts. Please start the login again.',
}

function errorText(error: unknown): string {
  if (error instanceof ApiError) {
    if (error.code === 'forbidden') return `⛔ Forbidden: ${error.message} (403)`
    return FRIENDLY[error.code] ?? error.message
  }
  return error instanceof Error ? error.message : 'Something went wrong.'
}

export function ErrorMessage({ error }: { error: unknown }) {
  if (!error) return null
  return (
    <p className="status status-error" role="alert">
      {errorText(error)}
    </p>
  )
}
