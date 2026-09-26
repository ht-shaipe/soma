// 工具箱 API：视频下载 / 字幕处理 / 剪映草稿 / 通知推送 / 数据导出 / 平台直连
import api, { extractData } from './index'

// ---- 视频下载（yt-dlp） ----

export function checkYtdlp() {
  return api.post('/download/check').then(extractData<{ installed: boolean; version: string }>)
}

export interface YtdlpInfo {
  id: string
  title: string
  duration: number
  uploader: string
  thumbnail: string
  formats: { formatId: string; ext: string; resolution: string; hasVideo: boolean; hasAudio: boolean }[]
}

export function getYtdlpInfo(url: string, proxy?: string) {
  return api.post('/download/info', { url, proxy: proxy || '' }).then(extractData<YtdlpInfo>)
}

export function downloadVideo(params: {
  url: string
  saveDir: string
  audioOnly?: boolean
  format?: string
  proxy?: string
}) {
  return api.post('/download/download', params).then(extractData<{ success: boolean; path: string; platform: string }>)
}

export function extractUrls(text: string) {
  return api
    .post('/download/extract_urls', { text })
    .then(extractData<{ urls: { url: string; platform: string }[] }>)
}

// ---- 字幕处理 ----

export interface SubtitleEntry {
  index: number
  startMs: number
  endMs: number
  text: string
}

export function parseSubtitle(path: string) {
  return api.post('/subtitle/parse', { path }).then(extractData<{ count: number; entries: SubtitleEntry[] }>)
}

export function convertSubtitle(inputPath: string, outputPath: string) {
  return api
    .post('/subtitle/convert', { inputPath, outputPath })
    .then(extractData<{ success: boolean; count: number; outputPath: string }>)
}

export function mergeSubtitles(paths: string[], outputPath: string) {
  return api
    .post('/subtitle/merge', { paths, outputPath })
    .then(extractData<{ success: boolean; count: number; outputPath: string }>)
}

export function translateSubtitle(inputPath: string, outputPath: string, targetLang: string, batchSize?: number) {
  return api
    .post('/subtitle/translate', { inputPath, outputPath, targetLang, batchSize })
    .then(extractData<{ success: boolean; count: number; outputPath: string }>)
}

export function correctSubtitle(inputPath: string, outputPath: string, correctType?: string) {
  return api
    .post('/subtitle/correct', { inputPath, outputPath, correctType })
    .then(extractData<{ success: boolean; count: number; outputPath: string }>)
}

// ---- 剪映草稿 ----

export function createJianyingDraft(params: {
  name: string
  videos: string[]
  audio?: string
  width?: number
  height?: number
  outputDir: string
}) {
  return api
    .post('/jianying/create', params)
    .then(extractData<{ success: boolean; draftPath: string; videoCount: number }>)
}

// ---- 通知推送 ----

export interface NotifySendResult {
  success: boolean
  channel: string
  message: string
}

export function sendNotify(params: {
  channel: string
  webhook: string
  title: string
  body: string
  secret?: string
  url?: string
}) {
  return api.post('/notify/send', params).then(extractData<NotifySendResult>)
}

// ---- 数据导出 ----

export function exportData(params: {
  outputPath: string
  data?: unknown[]
  inputPath?: string
  format?: string
  headers?: string[]
}) {
  return api
    .post('/dataexport/export', params)
    .then(extractData<{ success: boolean; count: number; outputPath: string }>)
}

export function importData(path: string) {
  return api
    .post('/dataexport/import', { path })
    .then(extractData<{ total: number; rows: Record<string, string>[] }>)
}

export function previewData(data: unknown[]) {
  return api
    .post('/dataexport/preview', { data })
    .then(extractData<{ total: number; rows: Record<string, string>[]; csvPreview: string }>)
}

// ---- 平台直连（抖音） ----

export interface DouyinDetail {
  awemeId: string
  desc: string
  createTime: number
  author: { nickname: string; secUid: string }
  statistics: { diggCount: number; commentCount: number; shareCount: number; collectCount: number; playCount: number }
  video: { cover: string; durationMs: number }
}

export function douyinDetail(awemeId: string, cookie?: string, proxy?: string) {
  return api
    .post('/platform/douyin_detail', { awemeId, cookie: cookie || '', proxy: proxy || '' })
    .then(extractData<DouyinDetail>)
}

export function douyinPosts(secUserId: string, count?: number, cursor?: number, cookie?: string, proxy?: string) {
  return api
    .post('/platform/douyin_posts', { secUserId, count, cursor, cookie: cookie || '', proxy: proxy || '' })
    .then(extractData<{ hasMore: boolean; cursor: number; count: number; posts: { awemeId: string; desc: string; createTime: number; diggCount: number; commentCount: number }[] }>)
}
