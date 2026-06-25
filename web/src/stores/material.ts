import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { MusicInfo, MaterialInfo } from '@/types'

export const useMaterialStore = defineStore('material', () => {
  const musics = ref<MusicInfo[]>([])
  const materials = ref<MaterialInfo[]>([])

  async function fetchMusics() {
    try {
      const { listMusics } = await import('@/api/music')
      musics.value = await listMusics()
    } catch {
      musics.value = []
    }
  }

  async function fetchMaterials() {
    try {
      const { listMaterials } = await import('@/api/material')
      materials.value = await listMaterials()
    } catch {
      materials.value = []
    }
  }

  return { musics, materials, fetchMusics, fetchMaterials }
})
