import { defineConfig } from 'vitest/config'
import vue from '@vitejs/plugin-vue'

export default defineConfig({
  plugins: [vue()],
  // Polling keeps file watching reliable on network / WSL mounted drives.
  server: { port: 5173, strictPort: true, watch: { usePolling: !!process.env.VITE_POLL } },
  build: { target: 'es2022' },
  test: {
    environment: 'happy-dom',
    include: ['tests/unit/**/*.spec.ts'],
    setupFiles: ['tests/unit/setup.ts'],
  },
})
