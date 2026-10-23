import { createApp } from 'vue'
import App from './App.vue'
import './styles/theme.css'
import { loadWasm } from './core/wasm'
import { createEditor, provideEditor } from './stores/editor'
import { prefs } from './stores/prefs'

async function boot(): Promise<void> {
  await loadWasm()
  const store = provideEditor(createEditor())
  store.engine.value.setHistoryLimit(prefs.historyLimit)
  store.refresh()
  // Handy for debugging and for end-to-end tests.
  ;(window as unknown as { __transit: unknown }).__transit = store
  createApp(App).mount('#app')
}

boot().catch((err) => {
  document.body.innerHTML = `<pre style="color:#f88;padding:20px">Failed to start: ${String(err)}</pre>`
})
