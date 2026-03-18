"use client"

import { api } from "@/api"
import { useEffect } from "react"

export default function Home() {

  useEffect(() => {
    async function load() {
      try {
        const data = await api.config.get()
        console.log(data)
      } catch (err) {
        console.error("Config fetch failed:", err)
      }
    }
    void load()
  }, [])

  return (
    <div className="flex min-h-screen items-center justify-center p-24">
      <h1 className="text-4xl font-bold">Certus Dashboard</h1>
    </div>
  )
}
