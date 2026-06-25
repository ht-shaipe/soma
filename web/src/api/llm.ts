import api, { extractData } from './index'

export function generateScript(videoSubject: string, videoLanguage?: string, paragraphNumber?: number, videoScriptPrompt?: string, customSystemPrompt?: string, useCustomSystemPrompt?: boolean) {
  return api.post('/scripts', {
    method: 'generate',
    video_subject: videoSubject,
    video_language: videoLanguage,
    paragraph_number: paragraphNumber,
    video_script_prompt: videoScriptPrompt,
    custom_system_prompt: customSystemPrompt,
    use_custom_system_prompt: useCustomSystemPrompt,
  }).then(extractData<{ script: string; terms: string }>)
}

export function generateTerms(videoScript: string, videoLanguage?: string) {
  return api.post('/terms', {
    method: 'generate',
    video_script: videoScript,
    video_language: videoLanguage,
  }).then(extractData<{ terms: string }>)
}

export function generateSocialMetadata(videoScript: string, videoSubject: string) {
  return api.post('/social', {
    method: 'generate',
    video_script: videoScript,
    video_subject: videoSubject,
  }).then(extractData<{ title: string; description: string; tags: string[] }>)
}
