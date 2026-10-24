import { expect, type Page } from '@playwright/test'

export async function openApp(page: Page): Promise<void> {
  // Native save/open pickers can't be driven by Playwright; use the download
  // and <input type=file> fallbacks unless a test installs its own picker.
  await page.addInitScript(() => {
    delete (window as any).showSaveFilePicker
    delete (window as any).showOpenFilePicker
  })
  await page.goto('/')
  await page.waitForSelector('[data-testid=canvas]')
  await page.evaluate(() => localStorage.clear())
  // Focus the canvas so single letter shortcuts reach the app.
  await page.click('[data-testid=canvas]', { position: { x: 5, y: 5 } })
}

/** Client coordinates of the center of a grid cell. */
export async function cellPos(page: Page, x: number, y: number): Promise<{ x: number; y: number }> {
  return page.evaluate(
    ([x, y]) => {
      const s = (window as any).__transit.state
      const host = document.querySelector('[data-testid=canvas]')!.getBoundingClientRect()
      const g = 24 // CELL_PX
      return {
        x: host.left + s.view.x + (x + 0.5) * g * s.view.zoom,
        y: host.top + s.view.y + (y + 0.5) * g * s.view.zoom,
      }
    },
    [x, y],
  )
}

export async function clickCell(page: Page, x: number, y: number, opts: Parameters<Page['mouse']['click']>[2] = {}) {
  const p = await cellPos(page, x, y)
  await page.mouse.click(p.x, p.y, opts)
}

export async function dragCells(page: Page, from: [number, number], to: [number, number], steps = 10) {
  const a = await cellPos(page, ...from)
  const b = await cellPos(page, ...to)
  await page.mouse.move(a.x, a.y)
  await page.mouse.down()
  await page.mouse.move(b.x, b.y, { steps })
  await page.mouse.up()
}

export async function drawLine(page: Page, pts: [number, number][]) {
  await page.keyboard.press('p')
  for (const [x, y] of pts) await clickCell(page, x, y)
  await page.keyboard.press('Enter')
}

export async function setPenColor(page: Page, hex: string) {
  const input = page.locator('[data-testid=options-bar] .color-field input.hex').first()
  await input.fill(hex)
  await input.press('Enter')
}

export function state<T>(page: Page, fn: (s: any) => T): Promise<T> {
  return page.evaluate(`(${fn.toString()})(window.__transit.state)`) as Promise<T>
}

export const fragmentKinds = (page: Page) => state(page, (s) => s.fragments.map((f: any) => f.kind))

export async function layerRow(page: Page, name: string) {
  const row = page.locator('[data-testid=layers-panel] .row').filter({ has: page.locator('.name', { hasText: new RegExp(`^${name}$`) }) })
  await expect(row.first()).toBeVisible()
  return row.first()
}

/** Build a small but complete map used by several tests. */
/** Drag a pictogram from the Icons panel onto a cell, then show the Properties tab again. */
export async function dropIcon(page: Page, icon: string, x: number, y: number) {
  await page.click('[data-testid=dock] [data-tab=Icons]')
  const target = await cellPos(page, x, y)
  const canvas = (await page.getByTestId('canvas').boundingBox())!
  await page.locator(`[data-testid=icons-panel] [data-icon=${icon}]`).dragTo(page.getByTestId('canvas'), {
    targetPosition: { x: target.x - canvas.x, y: target.y - canvas.y },
  })
  await page.click('[data-testid=dock] [data-tab=Properties]')
}

export async function buildSampleMap(page: Page) {
  await drawLine(page, [[-16, -4], [-6, -4], [0, 2], [12, 2]])
  await page.selectOption('[data-testid=line-style]', 'dashed')
  await setPenColor(page, '#0098d4')
  await drawLine(page, [[-2, -10], [-2, 9]])
  await page.selectOption('[data-testid=line-style]', 'hollow')
  await setPenColor(page, '#00843d')
  await drawLine(page, [[-14, 7], [-7, 7], [-4, 4], [6, 4], [6, -7]])
  await page.keyboard.press('s')
  await clickCell(page, -12, -4)
  await clickCell(page, -8, -4)
  await clickCell(page, 9, 2)
  await page.click('[data-testid=options-bar] button[role=radio]:has-text("Pill")')
  await dragCells(page, [-2, 2], [-2, 4])
  await page.click('[data-testid=options-bar] button[role=radio]:has-text("Tick")')
  await clickCell(page, -2, -7)
  await clickCell(page, 6, -3)
  await dropIcon(page, 'wheelchair', 14, 2)
  await page.keyboard.press('t')
  await clickCell(page, 0, -12)
  await page.keyboard.type('Metro Network')
  await page.keyboard.press('Enter')
  await page.keyboard.press('v')
}
