import { expect, test } from '@playwright/test'
import { clickCell, drawLine, openApp } from './helpers'

test.describe('workspace layout', () => {
  test.beforeEach(async ({ page }) => openApp(page))

  test('shows menu bar, tools, options, panels and status bar', async ({ page }) => {
    await expect(page).toHaveTitle('Transit Diagram Maker')
    for (const menu of ['File', 'Edit', 'Document', 'Layer', 'Select', 'View', 'Window', 'Help']) {
      await expect(page.locator(`[data-menu=${menu}]`)).toBeVisible()
    }
    for (const id of ['tools-panel', 'options-bar', 'properties-panel', 'layers-panel', 'status-bar', 'canvas']) {
      await expect(page.getByTestId(id)).toBeVisible()
    }
    const tools = page.locator('[data-testid=tools-panel] .tool')
    await expect(tools).toHaveCount(6)
    await expect(page.getByTestId('doc-title')).toHaveText(/^Untitled-1\.transit — /)
    // One document at a time: there is no document tab strip.
    await expect(page.locator('.doc-tabs')).toHaveCount(0)
  })

  test('tools panel is on the left, panels on the right', async ({ page }) => {
    const tools = await page.getByTestId('tools-panel').boundingBox()
    const canvas = await page.getByTestId('canvas').boundingBox()
    const layers = await page.getByTestId('layers-panel').boundingBox()
    expect(tools!.x).toBeLessThan(canvas!.x)
    expect(layers!.x).toBeGreaterThan(canvas!.x + canvas!.width - 1)
  })

  test('uses a dark theme', async ({ page }) => {
    const bg = await page.getByTestId('tools-panel').evaluate((el) => getComputedStyle(el).backgroundColor)
    const [r, g, b] = bg.match(/\d+/g)!.map(Number)
    expect(r + g + b).toBeLessThan(3 * 80)
  })

  test('window menu hides and shows panels', async ({ page }) => {
    await page.click('[data-menu=Window]')
    await page.click('.dropdown .item:has-text("Layers")')
    await expect(page.getByTestId('layers-panel')).toHaveCount(0)
    await page.click('[data-menu=Window]')
    await page.click('.dropdown .item:has-text("Layers")')
    await expect(page.getByTestId('layers-panel')).toBeVisible()
  })

  test('pen and eraser share a slot with a long-press flyout', async ({ page }) => {
    const slot = page.locator('[data-testid=tools-panel] .tool[data-tool=pen]')
    const box = (await slot.boundingBox())!
    await page.mouse.move(box.x + 10, box.y + 10)
    await page.mouse.down()
    await page.waitForTimeout(700)
    await page.mouse.up()
    const flyout = page.getByTestId('tool-flyout')
    await expect(flyout).toBeVisible()
    await flyout.locator('[data-tool=eraser]').click()
    await expect(page.locator('[data-testid=tools-panel] .tool.on')).toHaveAttribute('data-tool', 'eraser')
    // keyboard shortcut swaps the slot back
    await page.click('[data-testid=canvas]', { position: { x: 5, y: 5 } })
    await page.keyboard.press('p')
    await expect(page.locator('[data-testid=tools-panel] .tool.on')).toHaveAttribute('data-tool', 'pen')
  })

  test('long press, drag onto a flyout item and release picks it', async ({ page }) => {
    const slot = page.locator('[data-testid=tools-panel] .tool[data-tool=pen]')
    const box = (await slot.boundingBox())!
    await page.mouse.move(box.x + 10, box.y + 10)
    await page.mouse.down()
    await page.waitForTimeout(700)
    const item = (await page.getByTestId('tool-flyout').locator('[data-tool=eraser]').boundingBox())!
    await page.mouse.move(item.x + 20, item.y + item.height / 2, { steps: 5 })
    await page.mouse.up()
    await expect(page.getByTestId('tool-flyout')).toHaveCount(0)
    await expect(page.locator('[data-testid=tools-panel] .tool.on')).toHaveAttribute('data-tool', 'eraser')
  })

  test('double clicking the pen slot opens its flyout', async ({ page }) => {
    await page.locator('[data-testid=tools-panel] .tool[data-tool=pen]').dblclick()
    await expect(page.getByTestId('tool-flyout')).toBeVisible()
  })

  test('the properties panel keeps its size until the splitter is dragged', async ({ page }) => {
    const panel = page.locator('[data-panel=Properties]')
    const empty = (await panel.boundingBox())!.height
    await drawLine(page, [[0, 0], [6, 0]])
    await page.keyboard.press('v')
    await clickCell(page, 2, 0)
    await page.locator('[data-testid=properties-panel] [data-testid=prop-line-style]').selectOption('hollow_dashed')
    expect((await panel.boundingBox())!.height).toBe(empty)
    const split = (await page.getByTestId('dock-splitter').boundingBox())!
    await page.mouse.move(split.x + 50, split.y + 2)
    await page.mouse.down()
    await page.mouse.move(split.x + 50, split.y + 102, { steps: 4 })
    await page.mouse.up()
    expect((await panel.boundingBox())!.height).toBeCloseTo(empty + 100, 0)
    expect(await page.evaluate(() => JSON.parse(localStorage.getItem('transit-diagram-maker.prefs')!).propertiesHeight)).toBe(Math.round(empty + 100))
  })

  test('tool shortcuts switch tools', async ({ page }) => {
    const pairs: [string, string][] = [
      ['h', 'hand'],
      ['s', 'station'],
      ['t', 'text'],
      ['i', 'icon'],
      ['e', 'eraser'],
      ['v', 'select'],
    ]
    for (const [k, tool] of pairs) {
      await page.keyboard.press(k)
      await expect(page.locator('[data-testid=tools-panel] .tool.on')).toHaveAttribute('data-tool', tool)
    }
  })
})
