import axios from 'axios'

const api = axios.create({
  baseURL: '/api/v1',
  timeout: 120000,
  headers: { 'Content-Type': 'application/json' },
})

api.interceptors.response.use(
  (response) => {
    const data = response.data
    if (data && typeof data === 'object' && 'code' in data) {
      if (data.code !== 200) {
        return Promise.reject(new Error(data.message || data.msg || 'Request failed'))
      }
    }
    return response
  },
  (error) => {
    return Promise.reject(error)
  }
)

export function extractData<T>(response: { data: unknown }): T {
  const d = response.data
  if (d && typeof d === 'object' && 'code' in d && 'result' in d) {
    return (d as { code: number; result: T; message: string }).result
  }
  return d as T
}

export default api
