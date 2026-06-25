export function getStreamUrl(path: string) {
  return `/api/v1/stream/${encodeURIComponent(path)}`
}

export function getDownloadUrl(path: string) {
  return `/api/v1/download/${encodeURIComponent(path)}`
}
