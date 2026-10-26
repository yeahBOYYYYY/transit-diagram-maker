import { defineConfig } from 'vitest/config'
import vue from '@vitejs/plugin-vue'

export default defineConfig({
  plugins: [vue()],
  // Polling keeps file watching reliable on network / WSL mounted drives.
  server: { port: 5173, strictPort: true, watch: { usePolling: !!process.env.VITE_POLL } },
  build: { target: 'es2022' },
  test: {
    environment: 'happy-dom',
    // unit: one module at a time; component: Testing Library, driven the way a user would;
    // integration: several modules (tools, store, commands, engine, files) working together.
    include: ['tests/{unit,component,integration}/**/*.spec.ts'],
    setupFiles: ['tests/unit/setup.ts'],
  },
})
