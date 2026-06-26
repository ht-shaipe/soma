import api, { extractData } from './index'

export function generateScript(videoSubject: string, videoLanguage?: string, paragraphNumber?: number, videoScriptPrompt?: string, customSystemPrompt?: string, useCustomSystemPrompt?: boolean) {
  return api.post('/scripts/generate', {
    videoSubject,
    videoLanguage,
    paragraphNumber,
    videoScriptPrompt,
    customSystemPrompt,
    useCustomSystemPrompt,
  }).then(extractData<{ script: string }>)
}

export function generateTerms(videoScript: string, videoSubject?: string, videoLanguage?: string) {
  return api.post('/terms/generate', {
    videoScript,
    videoSubject,
    videoLanguage,
  }).then(extractData<{ terms: string[] }>)
}

export function generateSocialMetadata(videoScript: string, videoSubject: string, platform?: string) {
  return api.post('/social/generate', {
    videoScript,
    videoSubject,
    platform,
  }).then(extractData<{ title: string; description: string; tags: string[] }>)
}
