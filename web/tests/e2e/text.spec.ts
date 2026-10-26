import { expect, test, type Page } from '@playwright/test'
import { cellPos, clickCell, openApp, state } from './helpers'

/** Content of the only text box. */
const textBox = (page: Page) =>
  state(page, (s) => {
    const f = s.fragments.find((x: any) => x.kind === 'text')
    return f ? (window as any).__transit.node(f.id).content : null
  })

async function addText(page: Page, x: number, y: number, text: string) {
  await page.keyboard.press('t')
  await clickCell(page, x, y)
  await page.keyboard.type(text)
  await page.keyboard.press('Enter')
  await page.keyboard.press('v')
}

test.describe('text boxes', () => {
  test.beforeEach(async ({ page }) => openApp(page))

  test('parts of a text get their own formatting, like in a word processor', async ({ page }) => {
    await page.keyboard.press('t')
    await clickCell(page, 0, 0)
    await page.keyboard.type('Red Line')
    // Select "Red" and format it from the options bar and the keyboard.
    await page.keyboard.press('Home')
    for (let i = 0; i < 3; i++) await page.keyboard.press('Shift+ArrowRight')
    await page.getByTestId('text-bold').click()
    await page.keyboard.press('Control+i')
    const size = page.locator('[data-testid=text-size] input')
    await size.fill('1.5')
    await size.press('Tab')
    // A second paragraph, right-aligned on its own (the box is centered).
    await page.getByTestId('text-editor').click()
    await page.keyboard.press('End')
    await page.keyboard.press('Shift+Enter')
    await page.keyboard.type('Second')
    await page.click('[data-testid=text-align] [title=Right]')
    await page.keyboard.press('Enter')
    await expect(page.getByTestId('text-editor')).toHaveCount(0)
    const t = await textBox(page)
    expect(t.paragraphs).toEqual([
      { runs: [{ text: 'Red', bold: true, italic: true, size: 1.5 }, { text: ' Line' }] },
      { align: 'end', runs: [{ text: 'Second' }] },
    ])
    const svg = await state(page, (s) => s.fragments.find((f: any) => f.kind === 'text').svg)
    expect(svg).toContain('font-weight="bold" font-style="italic">Red</tspan> Line')
    // One undo step removes the whole new text.
    await page.keyboard.press('Control+z')
    expect(await textBox(page)).toBeNull()
  })

  test('the box is as wide as its text and changing the alignment does not move it', async ({ page }) => {
    await addText(page, 2, 2, 'Harbour Front')
    const t = await textBox(page)
    const measured = await page.evaluate(() => {
      const line = document.querySelector('[data-testid=doc-layer] text tspan[data-line]') as SVGTextContentElement
      return line.getComputedTextLength() / 24
    })
    expect(t.width).toBeCloseTo(measured, 2)
    const box = () => page.locator('[data-testid=doc-layer] text').boundingBox()
    const before = (await box())!
    for (const title of ['Left', 'Right', 'Center']) {
      await page.click(`[data-testid=prop-text-align] [title=${title}]`)
      const after = (await box())!
      expect(Math.abs(after.x - before.x), title).toBeLessThan(1)
      expect((await textBox(page)).pos, title).toEqual(t.pos)
    }
  })

  test('text snaps its nearest corner to a grid intersection; Alt moves and turns it freely', async ({ page }) => {
    await addText(page, 2, 2, 'Depot')
    const start = (await textBox(page)).pos
    expect(Number.isInteger(start.x) && Number.isInteger(start.y)).toBe(true)
    const a = await cellPos(page, start.x, start.y)
    // Drag by 1.3 cells: whichever corner of the box is nearest to an
    // intersection lands on it.
    await page.mouse.move(a.x, a.y)
    await page.mouse.down()
    await page.mouse.move(a.x + 1.3 * 24, a.y + 0.2 * 24, { steps: 6 })
    await page.mouse.up()
    const t = await textBox(page)
    const height = await state(page, (s) => s.fragments.find((f: any) => f.kind === 'text').bounds.h / 24)
    const onGrid = (v: number) => Math.abs(v - Math.round(v)) < 1e-6
    const corners = [0, t.width].flatMap((dx) => [0, height].map((dy) => [t.pos.x + dx, t.pos.y + dy]))
    expect(corners.some(([x, y]) => onGrid(x) && onGrid(y)), JSON.stringify({ pos: t.pos, w: t.width, height })).toBe(true)
    expect(Math.abs(t.pos.x - (start.x + 1.3))).toBeLessThan(1)
    expect(Math.abs(t.pos.y - (start.y + 0.2))).toBeLessThan(1)
    // With Alt it moves in tenths.
    const b = await cellPos(page, t.pos.x, t.pos.y)
    await page.keyboard.down('Alt')
    await page.mouse.move(b.x, b.y)
    await page.mouse.down()
    await page.mouse.move(b.x + 0.3 * 24, b.y, { steps: 6 })
    await page.mouse.up()
    await page.keyboard.up('Alt')
    const moved = (await textBox(page)).pos
    expect(moved.x).toBeCloseTo(t.pos.x + 0.3, 1)
    expect(moved.y).toBeCloseTo(t.pos.y, 1)
    // Rotate handles with Alt turn by any angle.
    const text = page.locator('[data-testid=doc-layer] text')
    await text.click()
    await expect(page.getByTestId('rotate-handle')).toHaveCount(4)
    const h = (await page.getByTestId('rotate-handle').nth(1).boundingBox())!
    await page.keyboard.down('Alt')
    await page.mouse.move(h.x + h.width / 2, h.y + h.height / 2)
    await page.mouse.down()
    await page.mouse.move(h.x + h.width / 2 + 4, h.y + h.height / 2 + 14, { steps: 6 })
    await page.mouse.up()
    await page.keyboard.up('Alt')
    const rotation = (await textBox(page)).rotation
    expect(rotation).not.toBe(0)
    expect(rotation % 45).not.toBe(0)
  })
})
