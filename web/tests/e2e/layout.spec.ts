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
    await expect(tools).toHaveCount(7)
    await expect(page.getByTestId('doc-title')).toHaveText(/^Untitled-1\.transit\.svg — /)
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

  test('the eraser has its own tool button and there is no icon tool', async ({ page }) => {
    const tools = page.locator('[data-testid=tools-panel] .tool')
    await expect(tools).toHaveCount(7)
    await expect(page.locator('[data-testid=tools-panel] .tool[data-tool=icon]')).toHaveCount(0)
    await page.locator('[data-testid=tools-panel] .tool[data-tool=eraser]').click()
    await expect(page.locator('[data-testid=tools-panel] .tool.on')).toHaveAttribute('data-tool', 'eraser')
    await expect(page.getByTestId('options-bar')).toContainText('Drag across a line')
    await page.locator('[data-testid=tools-panel] .tool[data-tool=pen]').click()
    await expect(page.locator('[data-testid=tools-panel] .tool.on')).toHaveAttribute('data-tool', 'pen')
  })

  test('the properties panel keeps its size until the splitter is dragged', async ({ page }) => {
    const panel = page.locator('[data-panel=Properties]')
    const empty = (await panel.boundingBox())!.height
    await drawLine(page, [[0, 0], [6, 0]])
    await page.keyboard.press('v')
    await clickCell(page, 2, 0)
    await page.locator('[data-testid=properties-panel] [data-testid=prop-line-style]').selectOption('inner_dashed')
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
      ['u', 'shape'],
      ['e', 'eraser'],
      ['v', 'select'],
    ]
    for (const [k, tool] of pairs) {
      await page.keyboard.press(k)
      await expect(page.locator('[data-testid=tools-panel] .tool.on')).toHaveAttribute('data-tool', tool)
    }
  })
})
