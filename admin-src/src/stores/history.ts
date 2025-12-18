import { storage } from "@/utils"
import { defineStore } from "pinia"


export const useHistoryStore = defineStore('history', {
  state: () => {
    return { 
      list: [],
     }
  },
  // could also be defined as
  // state: () => ({ count: 0 })
  actions: {
    append(item) {
      this.list.push(item)
    },
  },
  persist: {
    enabled: true,
    strategies: [
      {
        key: 'list',
        storage,
      },
    ],
  },
})