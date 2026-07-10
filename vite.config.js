import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

export default defineConfig({
  plugins: [
    react()
  ],

  server: {
    proxy: {
      '/api': {
        target: 'http://localhost:8489',
        changeOrigin: true,
        secure: false,
      },
      '/uploads': {
        target: 'http://localhost:8489',
        changeOrigin: true,
        secure: false,
      },
    },
  },

  build: {
    rollupOptions: {
      output: {
        // Rolldown (used by Vite 8) accepts a function here, not Rollup's
        // legacy object form. Keep large dependencies in stable cacheable chunks.
        manualChunks(id) {
          if (!id.includes('node_modules')) return undefined
          if (id.includes('lucide-react')) return 'icons'
          return 'vendor'
        },
      },
    },

    sourcemap: false,
    minify: 'oxc',
    chunkSizeWarningLimit: 1000,
  },
})
