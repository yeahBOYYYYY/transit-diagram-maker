// Visual regression: render a representative map and compare screenshots.
import { expect, test } from '@playwright/test'
import { buildSampleMap, openApp } from './helpers'

test.describe('visual', () => {
  test.beforeEach(async ({ page }) => openApp(page))

  test('empty workspace', async ({ page }) => {
    await expect(page).toHaveScreenshot('empty-workspace.png')
  })

  test('sample transit map', async ({ page }) => {
    await buildSampleMap(page)
    await page.keyboard.press('Control+d')
    await page.mouse.move(5, 5)
    await expect(page.getByTestId('canvas')).toHaveScreenshot('sample-map-canvas.png')
    await expect(page).toHaveScreenshot('sample-map-workspace.png')
  })

  test('exported SVG renders like the canvas', async ({ page }) => {
    await buildSampleMap(page)
    const svg = await page.evaluate(() => (window as any).__transit.engine.value.exportSvg())
    await page.setContent(`<body style="margin:0;background:#fff">${svg.replace(/^<\?xml[^>]*>/, '')}</body>`)
    await expect(page.locator('svg')).toHaveScreenshot('exported-svg.png')
  })

  test('menus and dialogs', async ({ page }) => {
    await page.click('[data-menu=Layer]')
    await expect(page.locator('.dropdown')).toHaveScreenshot('layer-menu.png')
    await page.keyboard.press('Escape')
    await page.click('[data-menu=Layer]')
    await page.keyboard.press('Control+Alt+d')
    await expect(page.getByTestId('dialog')).toHaveScreenshot('document-settings.png')
  })
})
