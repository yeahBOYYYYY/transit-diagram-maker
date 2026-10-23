import { expect, test } from '@playwright/test'
import { drawLine, openApp, state } from './helpers'

const view = (page: any) => state(page, (s) => ({ ...s.view }))

test.describe('infinite canvas navigation', () => {
  test.beforeEach(async ({ page }) => openApp(page))

  test('Ctrl + scroll zooms around the cursor', async ({ page }) => {
    await page.mouse.move(500, 400)
    const before = await view(page)
    await page.keyboard.down('Control')
    await page.mouse.wheel(0, -200)
    await page.keyboard.up('Control')
    const after = await view(page)
    expect(after.zoom).toBeGreaterThan(before.zoom)
    await expect(page.getByTestId('zoom-input')).not.toHaveValue('100')
  })

  test('hand tool, space-drag and scroll pan the canvas', async ({ page }) => {
    const start = await view(page)
    await page.keyboard.press('h')
    await page.mouse.move(600, 400)
    await page.mouse.down()
    await page.mouse.move(700, 450, { steps: 5 })
    await page.mouse.up()
    let v = await view(page)
    expect(v.x - start.x).toBeCloseTo(100)
    expect(v.y - start.y).toBeCloseTo(50)

    await page.keyboard.press('v')
    await page.keyboard.down('Space')
    await page.mouse.move(600, 400)
    await page.mouse.down()
    await page.mouse.move(550, 400, { steps: 5 })
    await page.mouse.up()
    await page.keyboard.up('Space')
    v = await view(page)
    expect(v.x - start.x).toBeCloseTo(50)

    await page.mouse.wheel(0, 120)
    const w = await view(page)
    expect(w.y).toBeLessThan(v.y)
  })

  test('the workspace is unbounded: draw far away and fit to screen', async ({ page }) => {
    await drawLine(page, [[0, 0], [3, 0]])
    // pan far away and draw another line there
    await page.evaluate(() => (window as any).__transit.panBy(-20000, -15000))
    const host = (await page.getByTestId('canvas').boundingBox())!
    await page.keyboard.press('p')
    await page.mouse.click(host.x + 300, host.y + 300)
    await page.mouse.click(host.x + 400, host.y + 300)
    await page.keyboard.press('Enter')
    const far = await state(page, (s) => (window as any).__transit.node(s.fragments[1].id).content.points[0])
    expect(far.x).toBeGreaterThan(500)
    await page.keyboard.press('Control+0')
    const v = await view(page)
    expect(v.zoom).toBeLessThan(0.1)
    await page.keyboard.press('Control+1')
    expect((await view(page)).zoom).toBe(1)
  })

  test('grid can be toggled from the view menu', async ({ page }) => {
    await expect(page.locator('rect[fill="url(#grid-minor)"]')).toHaveCount(1)
    await page.click('[data-menu=View]')
    await page.click('.dropdown .item:has-text("Show Grid")')
    await expect(page.locator('rect[fill="url(#grid-minor)"]')).toHaveCount(0)
  })
})
