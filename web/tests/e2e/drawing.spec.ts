import { expect, test } from '@playwright/test'
import { clickCell, cellPos, dragCells, drawLine, fragmentKinds, layerRow, openApp, setPenColor, state } from './helpers'

test.describe('drawing tools', () => {
  test.beforeEach(async ({ page }) => openApp(page))

  test('line pen draws a routed line in a new line group', async ({ page }) => {
    await page.keyboard.press('p')
    await clickCell(page, 0, 0)
    const hover = await cellPos(page, 6, 3)
    await page.mouse.move(hover.x, hover.y)
    await expect(page.locator('.overlay-world .preview path').first()).toBeAttached()
    await clickCell(page, 6, 3)
    await page.keyboard.press('Enter')
    expect(await fragmentKinds(page)).toEqual(['line'])
    await layerRow(page, 'Line 1')
    await layerRow(page, 'Segment 1')
    const pts = await state(page, (s) => {
      const id = s.fragments[0].id
      return (window as any).__transit.node(id).content.points
    })
    expect(pts).toEqual([{ x: 0, y: 0 }, { x: 3, y: 0 }, { x: 6, y: 3 }])
  })

  test('eraser cuts a line into separate segment layers', async ({ page }) => {
    await drawLine(page, [[0, 0], [12, 0]])
    await page.keyboard.press('e')
    await dragCells(page, [5, 0], [6, 0])
    expect(await fragmentKinds(page)).toEqual(['line', 'line'])
    await layerRow(page, 'Segment 2')
    await page.keyboard.press('Control+z')
    expect(await fragmentKinds(page)).toEqual(['line'])
  })

  test('station tool adds stations with labels to the line group', async ({ page }) => {
    await drawLine(page, [[0, 0], [12, 0]])
    await page.keyboard.press('s')
    await clickCell(page, 4, 0)
    expect(await fragmentKinds(page)).toEqual(['line', 'station', 'text'])
    await expect(page.locator('[data-testid=doc-layer] text')).toHaveText('Station 1')
    // off the line is rejected with a message
    await clickCell(page, 4, 8)
    await expect(page.getByTestId('status-message')).toContainText('line')
    expect(await fragmentKinds(page)).toHaveLength(3)
    // the station group is nested in the line group
    const nested = await state(page, (s) => s.tree[0].children.some((c: any) => c.role === 'station'))
    expect(nested).toBe(true)
  })

  test('tick and pill stations', async ({ page }) => {
    await drawLine(page, [[0, 0], [12, 0]])
    await drawLine(page, [[0, 1], [12, 1]])
    await page.keyboard.press('s')
    await page.click('[data-testid=options-bar] button[role=radio]:has-text("Tick")')
    await clickCell(page, 2, 0)
    await page.click('[data-testid=options-bar] button[role=radio]:has-text("Pill")')
    await dragCells(page, [8, 0], [8, 1])
    const shapes = await state(page, (s) =>
      s.fragments.filter((f: any) => f.kind === 'station').map((f: any) => (window as any).__transit.node(f.id).content),
    )
    expect(shapes.map((c: any) => c.shape)).toEqual(['tick', 'pill'])
    expect(shapes[1].line_ids).toHaveLength(2)
  })

  test('text tool types inline text', async ({ page }) => {
    await page.keyboard.press('t')
    await clickCell(page, 3, 3)
    const editor = page.getByTestId('text-editor')
    await expect(editor).toBeFocused()
    await page.keyboard.type('Harbour Front')
    await page.keyboard.press('Enter')
    await expect(editor).toHaveCount(0)
    await expect(page.locator('[data-testid=doc-layer] text')).toHaveText('Harbour Front')
    await layerRow(page, 'Harbour Front')
    // Escape on a new text discards it
    await clickCell(page, 8, 8)
    await page.keyboard.press('Escape')
    expect(await fragmentKinds(page)).toEqual(['text'])
  })

  test('clicking a pictogram in the Icons panel adds it in the middle of the view', async ({ page }) => {
    await page.click('[data-testid=dock] [data-tab=Icons]')
    await page.click('[data-testid=icons-panel] [data-icon=ferry]')
    await layerRow(page, 'Ferry')
    const icon = await state(page, (s) => (window as any).__transit.node(s.fragments[0].id).content)
    const middle = await state(page, (s) => ({
      x: Math.floor((s.viewport.width / 2 - s.view.x) / s.view.zoom / 24) + 0.5,
      y: Math.floor((s.viewport.height / 2 - s.view.y) / s.view.zoom / 24) + 0.5,
    }))
    expect(icon.pos).toEqual(middle)
    expect(icon.size).toBeCloseTo(1.24)
    expect(await state(page, (s) => s.tool)).toBe('select')
  })

  test('the eyedropper of a color field takes a color from the canvas', async ({ page }) => {
    await page.keyboard.press('p')
    await setPenColor(page, '#0098d4')
    await drawLine(page, [[0, 0], [6, 0]])
    await setPenColor(page, '#000000')
    const field = page.locator('[data-testid=options-bar] .color-field').first()
    await field.getByTestId('eyedropper').click()
    await expect(page.locator('body')).toHaveClass(/picking-color/)
    await clickCell(page, 3, 0)
    await expect(field.locator('input.hex')).toHaveValue('#0098d4')
    await expect(page.locator('body')).not.toHaveClass(/picking-color/)
    // The click only picked the color: the pen did not start a new line.
    expect(await fragmentKinds(page)).toEqual(['line'])
    expect(await state(page, (s) => s.overlay.preview)).toBe('')
  })

  test('dragged pictograms snap from cell to cell', async ({ page }) => {
    await page.click('[data-testid=dock] [data-tab=Icons]')
    const target = await cellPos(page, 2, 2)
    const canvas = (await page.getByTestId('canvas').boundingBox())!
    await page.locator('[data-testid=icons-panel] [data-icon=bus]').dragTo(page.getByTestId('canvas'), {
      targetPosition: { x: target.x - canvas.x, y: target.y - canvas.y },
    })
    await page.keyboard.press('v')
    const a = await cellPos(page, 2, 2)
    const b = await cellPos(page, 5, 4)
    await page.mouse.move(a.x, a.y)
    await page.mouse.down()
    // End a little off the cell center: the pictogram still lands on it.
    await page.mouse.move(b.x + 7, b.y - 6, { steps: 10 })
    await page.mouse.up()
    const pos = await state(page, (s) => (window as any).__transit.node(s.fragments[0].id).content.pos)
    expect(pos).toEqual({ x: 5.5, y: 4.5 })
  })

  test('pictograms can be dragged from the Icons panel onto the canvas', async ({ page }) => {
    await page.click('[data-testid=dock] [data-tab=Icons]')
    const target = await cellPos(page, 4, 3)
    const canvas = (await page.getByTestId('canvas').boundingBox())!
    await page.locator('[data-testid=icons-panel] [data-icon=bus]').dragTo(page.getByTestId('canvas'), {
      targetPosition: { x: target.x - canvas.x, y: target.y - canvas.y },
    })
    await layerRow(page, 'Bus')
    const icon = await state(page, (s) => (window as any).__transit.node(s.fragments[0].id).content)
    expect(icon.pos).toEqual({ x: 4.5, y: 3.5 })
    expect(await state(page, (s) => s.tool)).toBe('select')
  })

  test('line style options from the options bar are applied', async ({ page }) => {
    await page.keyboard.press('p')
    await page.selectOption('[data-testid=line-style]', 'dashed')
    await drawLine(page, [[0, 0], [8, 0]])
    const svg = await state(page, (s) => s.fragments[0].svg)
    expect(svg).toContain('stroke-dasharray')
  })
})
