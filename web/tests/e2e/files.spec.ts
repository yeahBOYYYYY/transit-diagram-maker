import { expect, test } from '@playwright/test'
import { readFileSync } from 'node:fs'
import { buildSampleMap, drawLine, fragmentKinds, openApp, state } from './helpers'

test.describe('saving, opening and exporting', () => {
  test.beforeEach(async ({ page }) => openApp(page))

  test('export SVG downloads a standalone drawing', async ({ page }) => {
    await buildSampleMap(page)
    await page.keyboard.press('Control+Shift+e')
    await expect(page.getByTestId('dialog')).toContainText('Export as SVG')
    const [download] = await Promise.all([page.waitForEvent('download'), page.getByTestId('dialog-ok').click()])
    expect(download.suggestedFilename()).toBe('Untitled-1.svg')
    const svg = readFileSync((await download.path())!, 'utf8')
    expect(svg).toMatch(/^<\?xml/)
    expect(svg).toContain('xmlns="http://www.w3.org/2000/svg"')
    expect(svg).toContain('stroke-dasharray')
    expect(svg).toContain('Metro Network')
    // the browser can parse it
    const ok = await page.evaluate((text) => {
      const d = new DOMParser().parseFromString(text, 'image/svg+xml')
      return d.documentElement.nodeName === 'svg' && !d.querySelector('parsererror')
    }, svg)
    expect(ok).toBe(true)
  })

  test('export options change the picture only and are remembered', async ({ page }) => {
    await drawLine(page, [[0, 0], [4, 0]])
    await page.keyboard.press('Control+Shift+e')
    const dialog = page.getByTestId('dialog')
    const scale = dialog.getByTestId('export-cell-px').locator('input')
    await expect(scale).toHaveValue('24')
    await scale.fill('48')
    await scale.press('Tab')
    await dialog.locator('label', { hasText: 'Transparent background' }).locator('input').check()
    const [download] = await Promise.all([page.waitForEvent('download'), page.getByTestId('dialog-ok').click()])
    const svg = readFileSync((await download.path())!, 'utf8')
    expect(svg).toContain('width="336" height="144"')
    expect(svg).not.toContain('<rect')
    expect(svg).not.toContain('transit:')
    // The document is unchanged; the dialog remembers the options.
    expect(await state(page, (s) => s.fragments[0].svg)).toContain('stroke-width="24"')
    expect(await state(page, (s) => s.canUndo)).toBe(true)
    await page.keyboard.press('Control+Shift+e')
    await expect(dialog.getByTestId('export-cell-px').locator('input')).toHaveValue('48')
    await expect(dialog.getByTestId('export-options')).not.toContainText('Line width')
  })

  test('save a .transit.svg file that is a picture and open it again losslessly', async ({ page }) => {
    await buildSampleMap(page)
    const before = await state(page, (s) => ({ tree: s.tree, frags: s.fragments.map((f: any) => f.svg) }))
    const [download] = await Promise.all([page.waitForEvent('download'), page.keyboard.press('Control+s')])
    expect(download.suggestedFilename()).toBe('Untitled-1.transit.svg')
    const path = (await download.path())!
    const text = readFileSync(path, 'utf8')
    // The saved file is an SVG picture of the map that also carries the editor data.
    const parsed = await page.evaluate((t) => {
      const d = new DOMParser().parseFromString(t, 'image/svg+xml')
      const ns = d.documentElement.lookupNamespaceURI('transit')
      return { root: d.documentElement.nodeName, ok: !d.querySelector('parsererror'), segments: d.getElementsByTagNameNS(ns, 'segment').length }
    }, text)
    const lines = (await fragmentKinds(page)).filter((k: string) => k === 'line').length
    expect(parsed).toEqual({ root: 'svg', ok: true, segments: lines })
    expect(text).toContain('Metro Network')

    page.once('dialog', (d) => d.accept())
    await page.keyboard.press('Control+Alt+n')
    expect(await fragmentKinds(page)).toEqual([])

    const [chooser] = await Promise.all([page.waitForEvent('filechooser'), page.keyboard.press('Control+o')])
    await chooser.setFiles({ name: 'city.transit.svg', mimeType: 'image/svg+xml', buffer: readFileSync(path) })
    await expect(page.getByTestId('doc-title')).toContainText('city.transit.svg')
    const after = await state(page, (s) => ({ tree: s.tree, frags: s.fragments.map((f: any) => f.svg) }))
    expect(after).toEqual(before)
  })

  test('the menu bar save button saves back to the same file after the first save', async ({ page }) => {
    await page.evaluate(() => {
      const w = window as any
      w.__writes = []
      w.__picks = 0
      w.showSaveFilePicker = async () => {
        w.__picks++
        return {
          name: 'metro.transit.svg',
          getFile: async () => new File([], 'metro.transit.svg'),
          createWritable: async () => ({ write: async (d: string) => void w.__writes.push(d), close: async () => {} }),
        }
      }
    })
    await page.keyboard.press('t')
    await page.mouse.click(700, 400)
    await page.keyboard.type('A')
    await page.keyboard.press('Enter')
    await page.getByTestId('quick-save').click()
    await expect(page.getByTestId('doc-title')).toContainText('metro.transit.svg')
    await page.keyboard.press('Control+s')
    await expect.poll(() => page.evaluate(() => (window as any).__writes.length)).toBe(2)
    expect(await page.evaluate(() => (window as any).__picks)).toBe(1)
  })

  test('the status bar links to the GitHub repository', async ({ page }) => {
    const link = page.locator('[data-testid=status-bar] [data-testid=github-link]')
    await expect(link).toHaveAttribute('href', 'https://github.com/yeahBOYYYYY/transit-diagram-maker')
    await expect(link).toHaveAttribute('target', '_blank')
  })

  test('uploaded fonts and SVG icons are stored in the document and exported', async ({ page }) => {
    // A font: upload it from the text tool's font list.
    await page.keyboard.press('t')
    const [fontChooser] = await Promise.all([
      page.waitForEvent('filechooser'),
      page.locator('[data-testid=options-bar] [data-testid=font-select]').selectOption({ label: 'Upload font…' }),
    ])
    await fontChooser.setFiles({ name: 'City_Sans.woff2', mimeType: 'font/woff2', buffer: Buffer.from('wOF2 fake font bytes') })
    await expect(page.locator('[data-testid=options-bar] [data-testid=font-select] option:checked')).toHaveText('City Sans')
    await page.mouse.click(700, 300)
    await page.keyboard.type('Central')
    await page.keyboard.press('Enter')

    // An icon: upload it in the Icons panel.
    await page.click('[data-testid=dock] [data-tab=Icons]')
    await expect(page.getByTestId('icons-panel')).toBeVisible()
    const svg = '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 10"><path d="M5 0L10 10H0Z" fill="currentColor" onclick="alert(1)"/><script>alert(1)</script></svg>'
    const [iconChooser] = await Promise.all([page.waitForEvent('filechooser'), page.getByTestId('upload-icon').click()])
    await iconChooser.setFiles({ name: 'star.svg', mimeType: 'image/svg+xml', buffer: Buffer.from(svg) })
    await page.locator('[data-testid=icons-panel] .cell[title=star]').click()
    const iconSvg = await state(page, (s) => s.fragments.find((f: any) => f.kind === 'icon').svg)
    expect(iconSvg).toContain('M5 0L10 10H0Z')
    expect(iconSvg).not.toContain('onclick')
    expect(iconSvg).not.toContain('script')

    // Both are listed in Document > Fonts & Icons…
    await page.click('[data-menu=Document]')
    await page.click('.dropdown .item:has-text("Fonts & Icons")')
    await expect(page.getByTestId('asset-fonts')).toContainText('City Sans')
    await expect(page.getByTestId('asset-icons')).toContainText('star')
    await page.getByTestId('dialog-ok').click()

    // Saved in the .transit.svg file and embedded in the SVG export.
    const saved = await page.evaluate(() => (window as any).__transit.engine.value.saveTransit())
    expect(saved).toContain('<style transit:font="City Sans">')
    expect(saved).toMatch(/<transit:icon name="custom-1" label="star"/)
    const exported = await page.evaluate(() => (window as any).__transit.engine.value.exportSvg())
    expect(exported).toContain('@font-face{font-family:"City Sans"')
  })

  test('opening an invalid file shows an error and keeps the document', async ({ page }) => {
    await buildSampleMap(page)
    const count = (await fragmentKinds(page)).length
    const [chooser] = await Promise.all([page.waitForEvent('filechooser'), page.keyboard.press('Control+o')])
    await chooser.setFiles({ name: 'broken.transit.svg', mimeType: 'image/svg+xml', buffer: Buffer.from('<svg xmlns="http://www.w3.org/2000/svg"/>') })
    await expect(page.getByTestId('status-message')).toContainText('no Transit Diagram Maker data')
    expect((await fragmentKinds(page)).length).toBe(count)
  })
})
