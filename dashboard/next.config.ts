import path from 'node:path'
import type { NextConfig } from 'next'

const nextConfig: NextConfig = {
  turbopack: {
    root: path.resolve(process.cwd(), '..'),
  },
  output: 'export',
}

export default nextConfig
