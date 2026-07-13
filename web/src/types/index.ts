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
  video_watermark?: string
  video_intro?: string
  video_outro?: string
  portrait_image?: string
  intent_style?: string
  intent_mood?: string
  intent_audience?: string
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
  Draft: 0,
  Completed: 1,
  Queued: 2,
  Paused: 3,
  Processing: 4,
} as const

export type TaskStateCode = typeof TaskStateCode[keyof typeof TaskStateCode]

export interface AiVideoSegmentLog {
  scene_id: number
  prompt: string
  status: 'pending' | 'submitted' | 'processing' | 'success' | 'failed' | 'timeout'
  message?: string
}

export interface TaskInfo {
  taskId: string
  state: number
  progress: number
  script?: string
  videoSubject?: string
  videoScript?: string
  terms?: string[]
  params?: VideoParams
  storyboard?: StoryboardScene[]
  narration?: string
  audioFile?: string
  audioDuration?: number
  subtitlePath?: string
  subtitleFile?: string
  materials?: string[]
  videos?: string[]
  combinedVideos?: string[]
  aiVideoLogs?: AiVideoSegmentLog[]
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

export function isTaskDraft(task: TaskInfo): boolean {
  return task.state === TaskStateCode.Draft
}

export function isTaskQueued(task: TaskInfo): boolean {
  return task.state === TaskStateCode.Queued
}

export function isTaskPaused(task: TaskInfo): boolean {
  return task.state === TaskStateCode.Paused
}

export function isTaskProcessing(task: TaskInfo): boolean {
  return task.state === TaskStateCode.Processing
}

export function getTaskStepLabel(task: TaskInfo): string {
  if (isTaskFailed(task)) return 'Failed'
  if (isTaskCompleted(task)) return 'Completed'
  if (isTaskQueued(task)) return 'Queued'
  if (isTaskPaused(task)) return 'Paused'
  if (isTaskDraft(task)) return 'Draft'
  const p = task.progress
  if (p < 5) return 'Intent'
  if (p < 15) return 'Script'
  if (p < 30) return 'Storyboard'
  if (p < 55) return 'Materials'
  if (p < 70) return 'Audio'
  if (p < 100) return 'Compose'
  return 'Completed'
}

export interface SubtitleCue {
  index: number
  startMs: number
  endMs: number
  text: string
}

export interface StoryboardScene {
  scene_id: number
  duration?: number
  narration: string
  visual_desc?: string
  visual_prompt: string
  camera_movement?: string
  transition?: string
  text_overlay?: string
  mood?: string
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

export interface FontInfo {
  name: string
  path: string
}

export interface AppConfig {
  app: {
    name: string
    version: string
    host: string
    port: number
    storage_path: string
    concurrent_tasks: number
    max_queued_tasks?: number
    video_source?: string
    video_codec?: string
    material_directory?: string
    edge_tts_timeout?: number
    endpoint?: string
    ffmpeg_path?: string
    imagemagick_path?: string
    subtitle_provider?: string
    enable_redis?: boolean
    redis_host?: string
    redis_port?: number
    redis_db?: number
    tls_verify?: boolean
  }
  llm: {
    provider: string
    model: string
    api_key: string
    base_url: string
    api_version?: string
    secret_key?: string
    account_id?: string
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
    gemini_key?: string
  }
  stock: {
    pexels_api_key: string
    pixabay_api_key: string
    coverr_api_key: string
  }
  aivideo?: {
    zhipu_video_api_key?: string
    zhipu_video_model?: string
    kling_access_key?: string
    kling_secret_key?: string
    kling_video_model?: string
    minimax_video_api_key?: string
    minimax_video_model?: string
    video_gen_timeout?: number
  }
  ffmpeg: {
    path: string
    threads: number
  }
  whisper: {
    provider: string
    model: string
    device?: string
    compute_type?: string
    endpoint?: string
  }
  proxy?: {
    http?: string
    https?: string
  }
  ui?: {
    hide_log?: boolean
    subtitle_position?: string
    custom_position?: number
    upload_post_enabled?: boolean
    upload_post_api_key?: string
    upload_post_username?: string
    upload_post_platforms?: string[]
    upload_post_auto_upload?: boolean
  }
}
