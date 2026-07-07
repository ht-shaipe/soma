import api, { extractData } from './index'
import type { StoryboardScene } from '@/types'

export interface IntentParams {
  intentStyle?: string
  intentMood?: string
  intentAudience?: string
}

export function generateScript(
  videoSubject: string,
  videoLanguage?: string,
  paragraphNumber?: number,
  videoScriptPrompt?: string,
  customSystemPrompt?: string,
  useCustomSystemPrompt?: boolean,
  intent?: IntentParams,
) {
  return api.post('/scripts/generate', {
    videoSubject,
    videoLanguage,
    paragraphNumber,
    videoScriptPrompt,
    customSystemPrompt,
    useCustomSystemPrompt,
    intentStyle: intent?.intentStyle,
    intentMood: intent?.intentMood,
    intentAudience: intent?.intentAudience,
  }).then(extractData<{ script: string; terms: string[] }>)
}

export function generateTerms(videoScript: string, videoSubject?: string, videoLanguage?: string) {
  return api.post('/terms/generate', {
    videoScript,
    videoSubject: videoSubject || '',
    videoLanguage: videoLanguage || '',
  }).then(extractData<{ terms: string[] }>)
}

export function generateSocialMetadata(videoScript: string, videoSubject: string, platform?: string) {
  return api.post('/social/generate', {
    videoScript,
    videoSubject,
    platform,
  }).then(extractData<{ title: string; description: string; tags: string[] }>)
}

export interface IntentResult {
  theme: string
  style: string
  duration: string
  aspect_ratio: string
  audience: string
  mood: string
  language: string
  platform: string
}

export function generateIntent(videoSubject: string, videoLanguage?: string, videoAspect?: string) {
  return api.post('/intent/generate', {
    videoSubject,
    videoLanguage: videoLanguage || '',
    videoAspect: videoAspect || '9:16',
  }).then(extractData<IntentResult>)
}

export function generateStoryboard(videoScript: string, videoSubject: string, clipDuration?: number, intentStyle?: string, intentMood?: string) {
  return api.post('/storyboard/generate', {
    videoScript,
    videoSubject,
    clipDuration: clipDuration || 3,
    intentStyle: intentStyle || '',
    intentMood: intentMood || '',
  }).then(extractData<StoryboardScene[]>)
}
