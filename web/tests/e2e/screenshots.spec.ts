// Screenshots for the README. Skipped in normal test runs; regenerate them
// with `npm run screenshots` after UI changes (they go to docs/images).
import { test, type Page } from '@playwright/test'
import { buildSampleMap, clickCell, openApp } from './helpers'

const OUT = '../docs/images'

test.skip(!process.env.SCREENSHOTS, 'run with npm run screenshots')

/** Real station names for the sample map's stations, in the order they are placed. */
const STATIONS = ['Shinjuku', 'Châtelet', 'Alexanderplatz', 'Grand Central', "King's Cross", 'Sants']
const LINES = ['Red Line', 'Blue Line', 'Green Line']
/** Each line's only segment, named after the stations at its ends. */
const SEGMENTS = ['Shinjuku to Alexanderplatz', "King's Cross to Grand Central", 'Grand Central to Sants']

/**
 * The sample map with stations from around the world, named layers and a
 * formatted title, framed in the middle of the canvas.
 */
async function sampleMap(page: Page) {
  await openApp(page)
  await buildSampleMap(page)
  await page.evaluate(
    ([stations, lines, segments]) => {
      const store = (window as any).__transit
      const walk = (items: any[], out: any[] = []): any[] => {
        for (const it of items) out.push(it, ...walk(it.children))
        return out
      }
      const nodes = walk(store.state.tree)
      store.run((e: any) => {
        stations.forEach((name, i) => {
          for (const n of nodes.filter((n) => n.name === `Station ${i + 1}`)) {
            if (n.kind === 'text') e.updateContent(n.id, { paragraphs: [{ runs: [{ text: name }] }] })
            else e.rename(n.id, name)
          }
        })
        lines.forEach((name, i) => {
          const group = nodes.find((n) => n.name === `Line ${i + 1}`)
          e.rename(group.id, name)
          e.rename(group.children.find((c: any) => c.kind === 'line').id, segments[i])
        })
        const title = store.state.fragments.find((f: any) => f.kind === 'text' && store.node(f.id).content.station_id === null)
        e.updateContent(title.id, {
          paragraphs: [
            { runs: [{ text: 'World', bold: true, size: 1.4 }, { text: ' Metro', size: 1.4, color: '#0098d4' }] },
            { runs: [{ text: 'Made with Transit Diagram Maker', italic: true, color: '#666666' }] },
          ],
        })
      })
    },
    [STATIONS, LINES, SEGMENTS],
  )
  // Once the new names are measured, move two labels clear of the lines: the
  // interchange's to the left of its pill, between the red and green lines,
  // and Alexanderplatz's below its station, away from the green line.
  await page.waitForTimeout(100)
  await page.evaluate(
    ([grand, alex]) => {
      const store = (window as any).__transit
      const label = (name: string) => {
        const f = store.state.fragments.find((f: any) => f.kind === 'text' && store.node(f.id).name === name)
        return { id: f.id, t: store.node(f.id).content }
      }
      const g = label(grand)
      const a = label(alex)
      store.run((e: any) => {
        e.updateContent(g.id, { align: 'end', pos: { x: -2.4 - g.t.width, y: 3.5 - (g.t.size * 1.2) / 2 } })
        e.updateContent(a.id, { pos: { x: 9.5 - a.t.width / 2, y: 3.3 } })
      })
      store.clearSelection()
      store.fitToScreen()
    },
    [STATIONS[3], STATIONS[2]],
  )
}

test('workspace', async ({ page }) => {
  await sampleMap(page)
  await clickCell(page, -10, -4)
  await page.screenshot({ path: `${OUT}/workspace.png` })
})

test('rich text editing', async ({ page }) => {
  await sampleMap(page)
  const title = page.locator('[data-testid=doc-layer] text').filter({ hasText: 'World' })
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
