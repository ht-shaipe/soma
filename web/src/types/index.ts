export interface VideoParams {
  video_subject: string
  video_script?: string
  video_terms?: string | string[]
  video_aspect?: string
  video_concat_mode?: string
  video_transition_mode?: string
  video_clip_duration?: number
  video_count?: number
  video_source?: string
  voice_name?: string
  voice_volume?: number
  voice_rate?: number
  bgm_type?: string
  bgm_file?: string
  bgm_volume?: number
  subtitle_enabled?: boolean
  subtitle_position?: string
  custom_position?: number
  font_name?: string
  font_size?: number
  text_fore_color?: string
  stroke_color?: string
  stroke_width?: number
  text_background_color?: boolean | string
  rounded_subtitle_background?: boolean
  paragraph_number?: number
  video_script_prompt?: string
  custom_system_prompt?: string
  use_custom_system_prompt?: boolean
  match_materials_to_script?: boolean
  video_language?: string
  n_threads?: number
  video_encoder?: string
  custom_audio_file?: string
  video_materials?: MaterialInfo[]
}

export interface MaterialInfo {
  name: string
  path: string
  size?: number
  type?: string
  url?: string
  provider?: string
  duration?: number
  width?: number
  height?: number
}

export const TaskStateCode = {
  Failed: -1,
  Completed: 1,
  Processing: 4,
} as const

export type TaskStateCode = typeof TaskStateCode[keyof typeof TaskStateCode]

export interface TaskInfo {
  taskId: string
  state: number
  progress: number
  script?: string
  terms?: string[]
  audioFile?: string
  audioDuration?: number
  subtitlePath?: string
  materials?: string[]
  videos?: string[]
  combinedVideos?: string[]
  errorMessage?: string
  createdAt: string
  updatedAt: string
}

export function isTaskCompleted(task: TaskInfo): boolean {
  return task.state === TaskStateCode.Completed
}

export function isTaskFailed(task: TaskInfo): boolean {
  return task.state === TaskStateCode.Failed
}

export function isTaskProcessing(task: TaskInfo): boolean {
  return task.state === TaskStateCode.Processing
}

export function getTaskStepLabel(task: TaskInfo): string {
  if (isTaskFailed(task)) return 'Failed'
  if (isTaskCompleted(task)) return 'Completed'
  const p = task.progress
  if (p < 10) return 'Script'
  if (p < 20) return 'Terms'
  if (p < 40) return 'Audio'
  if (p < 60) return 'Subtitle'
  if (p < 80) return 'Materials'
  if (p < 100) return 'Video'
  return 'Completed'
}

export interface SubtitleCue {
  index: number
  startMs: number
  endMs: number
  text: string
}

export interface ApiResponse<T = unknown> {
  code: number
  message: string
  result: T
}

export interface LlmProviderOption {
  label: string
  value: string
  defaultBaseUrl?: string
  defaultModel?: string
  needApiKey?: boolean
  needSecretKey?: boolean
  needAccountId?: boolean
  tip?: string
}

export interface VoiceOption {
  label: string
  value: string
  locale?: string
  gender?: string
}

export interface MusicInfo {
  name: string
  file?: string
  path: string
  size?: number
}

export interface AppConfig {
  app: {
    name: string
    version: string
    host: string
    port: number
    storage_path: string
    concurrent_tasks: number
  }
  llm: {
    provider: string
    model: string
    api_key: string
    base_url: string
    api_version?: string
  }
  tts: {
    provider: string
    voice_name: string
    azure_speech_key?: string
    azure_speech_region?: string
    siliconflow_key?: string
    elevenlabs_key?: string
    elevenlabs_model?: string
    mimo_key?: string
  }
  stock: {
    pexels_api_key: string
    pixabay_api_key: string
    coverr_api_key: string
  }
  ffmpeg: {
    path: string
    threads: number
  }
  whisper: {
    provider: string
    model: string
    endpoint?: string
  }
}
