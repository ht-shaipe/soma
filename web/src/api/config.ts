// 系统配置 API：全局配置读取与保存
import api, { extractData } from './index'
import type { AppConfig } from '@/types'

export function getConfig() {
  return api.post('/config/get').then(extractData<AppConfig>)
}

export function saveConfig(config: Partial<AppConfig>) {
  return api.post('/config/save', config).then(extractData<boolean>)
}
