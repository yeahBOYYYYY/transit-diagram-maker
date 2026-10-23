import { expect, test } from '@playwright/test'
import { readFileSync } from 'node:fs'
import { buildSampleMap, fragmentKinds, openApp, state } from './helpers'

test.describe('saving, opening and exporting', () => {
  test.beforeEach(async ({ page }) => openApp(page))

  test('export SVG downloads a standalone drawing', async ({ page }) => {
    await buildSampleMap(page)
    const [download] = await Promise.all([page.waitForEvent('download'), page.keyboard.press('Control+Shift+e')])
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

  test('opening an invalid file shows an error and keeps the document', async ({ page }) => {
    await buildSampleMap(page)
    const count = (await fragmentKinds(page)).length
    const [chooser] = await Promise.all([page.waitForEvent('filechooser'), page.keyboard.press('Control+o')])
    await chooser.setFiles({ name: 'broken.transit', mimeType: 'application/json', buffer: Buffer.from('{"nope":1}') })
    await expect(page.getByTestId('status-message')).toContainText('could not read file')
    expect((await fragmentKinds(page)).length).toBe(count)
  })
})
