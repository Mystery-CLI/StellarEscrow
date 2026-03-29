import { fetchBaseQuery } from '@reduxjs/toolkit/query/react'

export interface NormalizedError {
  status: number | string
  message: string
  data?: unknown
}

interface RetryConfig {
  retries?: number
  backoffMs?: number
}

interface BaseQueryConfig {
  baseUrl?: string
  timeoutMs?: number
  retry?: RetryConfig
}

export const createBaseQuery = (config: BaseQueryConfig = {}) => {
  const {
    baseUrl = '/api',
    timeoutMs = 30000,
    retry = { retries: 3, backoffMs: 500 },
  } = config

  const rawBaseQuery = fetchBaseQuery({ baseUrl })

  return async (args: any, api: any, extraOptions: any) => {
    let attempt = 0

    while (true) {
      const controller = new AbortController()
      const timeout = setTimeout(() => controller.abort(), timeoutMs)

      try {
        const requestArgs =
          typeof args === 'string'
            ? { url: args, signal: controller.signal }
            : { ...args, signal: controller.signal }

        const result = await rawBaseQuery(requestArgs, api, extraOptions)

        clearTimeout(timeout)

        if (!result.error) return result

        const status = result.error.status
        const shouldRetry =
          typeof status === 'number' &&
          [408, 429, 500, 502, 503, 504].includes(status)

        if (!shouldRetry || attempt >= (retry.retries ?? 3)) {
          return {
            error: normalizeError(result.error),
          }
        }

        await delay((retry.backoffMs ?? 500) * 2 ** attempt)
        attempt++
      } catch (err: any) {
        clearTimeout(timeout)

        if (err.name === 'AbortError') {
          return {
            error: {
              status: 'TIMEOUT_ERROR',
              message: `Request timed out after ${timeoutMs}ms`,
            },
          }
        }

        return {
          error: {
            status: 'FETCH_ERROR',
            message: 'Failed to fetch',
          },
        }
      }
    }
  }
}

function normalizeError(error: any): NormalizedError {
  if (typeof error.status === 'number') {
    return {
      status: error.status,
      message: `HTTP ${error.status}`,
      data: error.data,
    }
  }

  return {
    status: 'FETCH_ERROR',
    message: 'Unknown error',
  }
}

function delay(ms: number) {
  return new Promise((res) => setTimeout(res, ms))
}