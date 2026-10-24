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

  test('export options apply to one export only', async ({ page }) => {
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
    // The document keeps its own settings.
    expect(await state(page, (s) => s.settings.export_background)).toBe('#ffffff')
    expect(await state(page, (s) => s.fragments[0].svg)).toContain('stroke-width="24"')
  })

  test('save .transit and open it again losslessly', async ({ page }) => {
    await buildSampleMap(page)
    const before = await state(page, (s) => ({ tree: s.tree, frags: s.fragments.map((f: any) => f.svg) }))
    const [download] = await Promise.all([page.waitForEvent('download'), page.keyboard.press('Control+s')])
    expect(download.suggestedFilename()).toBe('Untitled-1.transit')
    const path = (await download.path())!
    const json = JSON.parse(readFileSync(path, 'utf8'))
    expect(json.format).toBe('transit-diagram')

    page.once('dialog', (d) => d.accept())
    await page.keyboard.press('Control+Alt+n')
    expect(await fragmentKinds(page)).toEqual([])

    const [chooser] = await Promise.all([page.waitForEvent('filechooser'), page.keyboard.press('Control+o')])
    await chooser.setFiles({ name: 'city.transit', mimeType: 'application/json', buffer: readFileSync(path) })
    await expect(page.getByTestId('doc-title')).toContainText('city.transit')
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
          name: 'metro.transit',
          getFile: async () => new File([], 'metro.transit'),
          createWritable: async () => ({ write: async (d: string) => void w.__writes.push(d), close: async () => {} }),
        }
      }
    })
    await page.keyboard.press('t')
    await page.mouse.click(700, 400)
    await page.keyboard.type('A')
    await page.keyboard.press('Enter')
    await page.getByTestId('quick-save').click()
    await expect(page.getByTestId('doc-title')).toContainText('metro.transit')
    await page.keyboard.press('Control+s')
    await expect.poll(() => page.evaluate(() => (window as any).__writes.length)).toBe(2)
    expect(await page.evaluate(() => (window as any).__picks)).toBe(1)
  })

  test('the menu bar links to the GitHub repository', async ({ page }) => {
    const link = page.getByTestId('github-link')
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

    // An icon: upload it from the pictogram picker.
    await page.keyboard.press('i')
    await page.getByTestId('icon-picker').click()
    const svg = '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 10"><path d="M5 0L10 10H0Z" fill="currentColor" onclick="alert(1)"/><script>alert(1)</script></svg>'
    const [iconChooser] = await Promise.all([page.waitForEvent('filechooser'), page.getByTestId('upload-icon').click()])
    await iconChooser.setFiles({ name: 'star.svg', mimeType: 'image/svg+xml', buffer: Buffer.from(svg) })
    await expect(page.getByTestId('icon-picker')).toContainText('star')
    await page.mouse.click(700, 450)
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

    // Saved in the .transit file and embedded in the SVG export.
    const saved = JSON.parse(await page.evaluate(() => (window as any).__transit.engine.value.saveTransit()))
    expect(saved.document.assets.fonts[0].family).toBe('City Sans')
    expect(saved.document.assets.icons[0].label).toBe('star')
    const exported = await page.evaluate(() => (window as any).__transit.engine.value.exportSvg())
    expect(exported).toContain('@font-face{font-family:"City Sans"')
  })

  test('opening an invalid file shows an error and keeps the document', async ({ page }) => {
    await buildSampleMap(page)
    const count = (await fragmentKinds(page)).length
    const [chooser] = await Promise.all([page.waitForEvent('filechooser'), page.keyboard.press('Control+o')])
    await chooser.setFiles({ name: 'broken.transit', mimeType: 'application/json', buffer: Buffer.from('{"nope":1}') })
    await expect(page.getByTestId('status-message')).toContainText('could not read file')
    expect((await fragmentKinds(page)).length).toBe(count)
  })
})
