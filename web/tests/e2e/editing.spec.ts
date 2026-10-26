import { expect, test } from '@playwright/test'
import { cellPos, clickCell, dragCells, drawLine, fragmentKinds, layerRow, openApp, state } from './helpers'

const firstPoint = (page: any) => state(page, (s) => (window as any).__transit.node(s.fragments[0].id).content.points[0])

test.describe('selection, transforms and history', () => {
  test.beforeEach(async ({ page }) => openApp(page))

  test('select and move a line, then undo and redo', async ({ page }) => {
    await drawLine(page, [[0, 0], [6, 0]])
    await page.keyboard.press('v')
    await clickCell(page, 2, 0)
    await expect(page.getByTestId('selection-rect')).toHaveCount(1)
    await dragCells(page, [2, 0], [2, 3])
    expect(await firstPoint(page)).toEqual({ x: 0, y: 3 })
    await page.keyboard.press('Control+z')
    expect(await firstPoint(page)).toEqual({ x: 0, y: 0 })
    await page.keyboard.press('Control+Shift+z')
    expect(await firstPoint(page)).toEqual({ x: 0, y: 3 })
  })

  test('clicking a selected object reveals rotate handles; dragging rotates', async ({ page }) => {
    await drawLine(page, [[0, 0], [6, 0]])
    await page.keyboard.press('v')
    await page.keyboard.press('Control+d')
    await clickCell(page, 3, 0) // select
    await expect(page.getByTestId('rotate-handle')).toHaveCount(0)
    await clickCell(page, 3, 0) // click again: rotate mode
    const handles = page.getByTestId('rotate-handle')
    await expect(handles).toHaveCount(4)
    const h = (await handles.nth(1).boundingBox())!
    const center = await cellPos(page, 3, 0)
    await page.mouse.move(h.x + h.width / 2, h.y + h.height / 2)
    await page.mouse.down()
    // swing the handle a quarter turn around the center
    const r = Math.hypot(h.x - center.x, h.y - center.y)
    await page.mouse.move(center.x + r * 0.7, center.y + r * 0.7, { steps: 6 })
    await page.mouse.move(center.x + 5, center.y + r, { steps: 6 })
    await page.mouse.up()
    const pts = await state(page, (s) => (window as any).__transit.node(s.fragments[0].id).content.points)
    expect(pts[0].x).toBe(pts[1].x) // now vertical
  })

  test('marquee selects several layers and group/ungroup works', async ({ page }) => {
    await drawLine(page, [[0, 0], [4, 0]])
    await drawLine(page, [[0, 3], [4, 3]])
    await page.keyboard.press('v')
    await dragCells(page, [-2, -2], [6, 5])
    await expect(page.getByTestId('selection-rect')).toHaveCount(2)
    await page.keyboard.press('Control+g')
    await layerRow(page, 'Group 1')
    await page.keyboard.press('Control+Shift+g')
    await expect(page.locator('[data-testid=layers-panel] .name', { hasText: 'Group 1' })).toHaveCount(0)
  })

  test('delete, duplicate and nudge with the keyboard', async ({ page }) => {
    await drawLine(page, [[0, 0], [4, 0]])
    await page.keyboard.press('v')
    await clickCell(page, 1, 0)
    await page.keyboard.press('ArrowDown')
    expect(await firstPoint(page)).toEqual({ x: 0, y: 1 })
    await page.keyboard.press('Control+j')
    expect(await fragmentKinds(page)).toEqual(['line', 'line'])
    await page.keyboard.press('Delete')
    expect(await fragmentKinds(page)).toEqual(['line'])
  })

  test('dragging a station keeps it on its line and its label follows', async ({ page }) => {
    await drawLine(page, [[0, 0], [10, 0]])
    await page.keyboard.press('s')
    await clickCell(page, 2, 0)
    await page.keyboard.press('v')
    await dragCells(page, [2, 0], [7, 3])
    const st = await state(page, (s) => {
      const t = (window as any).__transit
      const station = s.fragments.find((f: any) => f.kind === 'station')
      const label = s.fragments.find((f: any) => f.kind === 'text')
      const box = t.node(label.id).content
      return { cell: t.node(station.id).content.cell, labelCenter: box.pos.x + box.width / 2 }
    })
    expect(st.cell).toEqual({ x: 7, y: 0 })
    expect(st.labelCenter, 'the label stays centered over the station').toBeCloseTo(7.5, 1)
  })

  test('a selected pill grows into the cell of a double-clicked arrow', async ({ page }) => {
    await drawLine(page, [[0, 0], [8, 0]])
    await drawLine(page, [[0, 2], [8, 2]])
    await page.keyboard.press('s')
    await page.click('[data-testid=options-bar] button[role=radio]:has-text("Pill")')
    await clickCell(page, 4, 0) // creates and selects the pill
    await page.keyboard.press('v')
    await expect(page.getByTestId('pill-grow')).toHaveCount(8)
    await page.locator('[data-testid=pill-grow][data-cell="4,1"]').dblclick()
    await page.locator('[data-testid=pill-grow][data-cell="4,2"]').dblclick()
    const st = await state(page, (s) => {
      const f = s.fragments.find((x: any) => x.kind === 'station')
      return (window as any).__transit.node(f.id).content
    })
    expect(st.cells).toEqual([{ x: 4, y: 0 }, { x: 4, y: 1 }, { x: 4, y: 2 }])
    expect(st.line_ids).toHaveLength(2)
    await expect(page.getByTestId('pill-grow')).toHaveCount(12)
  })

  test('double click edits text in place', async ({ page }) => {
    await page.keyboard.press('t')
    await clickCell(page, 0, 0)
    await page.keyboard.type('Old')
    await page.keyboard.press('Enter')
    await page.keyboard.press('v')
    await page.locator('[data-testid=doc-layer] text').dblclick()
    await expect(page.getByTestId('text-editor')).toBeVisible()
    await page.keyboard.press('Control+a')
    await page.keyboard.type('New')
    await page.keyboard.press('Enter')
    await expect(page.locator('[data-testid=doc-layer] text')).toHaveText('New')
  })

  test('properties panel edits a line style for all segments', async ({ page }) => {
    await drawLine(page, [[0, 0], [10, 0]])
    await page.keyboard.press('e')
    await clickCell(page, 5, 0)
    await page.keyboard.press('v')
    await clickCell(page, 1, 0)
    await page.locator('[data-testid=properties-panel] [data-testid=prop-line-style]').selectOption('hollow')
    const hollow = await state(page, (s) => s.fragments.every((f: any) => f.svg.split('<path').length - 1 === 4))
    expect(hollow).toBe(true)
  })
})
