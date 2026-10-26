// Screenshots for the README. Skipped in normal test runs; regenerate them
// with `npm run screenshots` after UI changes (they go to docs/images).
import { test, type Page } from '@playwright/test'
import { buildSampleMap, clickCell, openApp } from './helpers'

const OUT = '../docs/images'

test.skip(!process.env.SCREENSHOTS, 'run with npm run screenshots')

/** The sample map with a formatted title, framed in the middle of the canvas. */
async function sampleMap(page: Page) {
  await openApp(page)
  await buildSampleMap(page)
  await page.evaluate(() => {
    const store = (window as any).__transit
    const title = store.state.fragments.find((f: any) => f.kind === 'text' && store.node(f.id).content.station_id === null)
    store.run((e: any) =>
      e.updateContent(title.id, {
        paragraphs: [
          { runs: [{ text: 'Metro', bold: true, size: 1.4 }, { text: ' Network', size: 1.4, color: '#0098d4' }] },
          { runs: [{ text: 'Made with Transit Diagram Maker', italic: true, color: '#666666' }] },
        ],
      }),
    )
    store.clearSelection()
    store.fitToScreen()
  })
}

test('workspace', async ({ page }) => {
  await sampleMap(page)
  await clickCell(page, -10, -4)
  await page.screenshot({ path: `${OUT}/workspace.png` })
})

test('rich text editing', async ({ page }) => {
  await sampleMap(page)
  const title = page.locator('[data-testid=doc-layer] text').filter({ hasText: 'Metro' })
  await title.dblclick()
  await page.keyboard.press('Control+Home')
  for (let i = 0; i < 5; i++) await page.keyboard.press('Shift+ArrowRight')
  await page.screenshot({ path: `${OUT}/rich-text.png` })
})

test('icons panel', async ({ page }) => {
  await sampleMap(page)
  await page.click('[data-testid=dock] [data-tab=Icons]')
  await page.screenshot({ path: `${OUT}/icons-panel.png` })
})

test('preferences', async ({ page }) => {
  await sampleMap(page)
  await page.keyboard.press('Control+k')
  await page.getByTestId('prefs-tab-canvas').click()
  await page.screenshot({ path: `${OUT}/preferences.png` })
})

test('exported map', async ({ page }) => {
  await sampleMap(page)
  const svg = await page.evaluate(() => (window as any).__transit.engine.value.exportSvgWith({ cell_px: 24, margin: 1, background: '#ffffff' }))
  const fs = await import('node:fs')
  fs.writeFileSync(`${OUT}/sample-map.svg`, svg)
})
