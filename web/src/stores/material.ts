import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { MusicInfo, MaterialInfo } from '@/types'

export const useMaterialStore = defineStore('material', () => {
  const musics = ref<MusicInfo[]>([])
  const materials = ref<MaterialInfo[]>([])

  async function fetchMusics() {
    try {
      const { listMusics } = await import('@/api/music')
      const result = await listMusics()
      musics.value = result.list
    } catch (e) {
      musics.value = []
      console.error('Failed to fetch musics:', e)
    }
  }

  async function fetchMaterials() {
    try {
      const { listMaterials } = await import('@/api/material')
      const result = await listMaterials()
      materials.value = result.list
    } catch (e) {
      materials.value = []
      console.error('Failed to fetch materials:', e)
    }
  }

  return { musics, materials, fetchMusics, fetchMaterials }
})
