import axios from 'axios'
import type { ApiResponse } from '@/types'

const api = axios.create({
  baseURL: '/api/v1',
  timeout: 120000,
  headers: { 'Content-Type': 'application/json' },
})

api.interceptors.response.use(
  (response) => {
    const data = response.data as ApiResponse
    if (data.code !== undefined && data.code !== 0) {
      return Promise.reject(new Error(data.msg || 'Request failed'))
    }
    return response
  },
  (error) => {
    return Promise.reject(error)
  }
)

export function extractData<T>(response: { data: ApiResponse<T> | T }): T {
  const d = response.data
  if (d && typeof d === 'object' && 'code' in d && 'data' in d) {
    return (d as ApiResponse<T>).data
  }
  return d as T
}

export default api
