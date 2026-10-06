import { useCallback, useState } from 'react'

interface AsyncState<T> {
  data: T | undefined
  error: unknown
  loading: boolean
}

/**
 * One pattern for every API call: `run(...)` tracks loading/error/data.
 * `run` resolves to the data, or `undefined` on failure (the error is kept in state).
 */
export function useAsync<T, A extends unknown[]>(fn: (...args: A) => Promise<T>, initialLoading = false) {
  const [state, setState] = useState<AsyncState<T>>({ data: undefined, error: undefined, loading: initialLoading })

  const run = useCallback(
    async (...args: A): Promise<T | undefined> => {
      setState((s) => ({ ...s, loading: true, error: undefined }))
      try {
        const data = await fn(...args)
        setState({ data, error: undefined, loading: false })
        return data
      } catch (error) {
        setState((s) => ({ ...s, error, loading: false }))
        return undefined
      }
    },
    [fn],
  )

  const reset = useCallback(() => setState({ data: undefined, error: undefined, loading: false }), [])

  return { ...state, run, reset }
}
