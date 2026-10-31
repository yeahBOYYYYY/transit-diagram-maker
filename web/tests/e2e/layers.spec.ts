import { expect, test } from '@playwright/test'
import { clickCell, drawLine, fragmentKinds, layerRow, openApp, state } from './helpers'

test.describe('layers panel', () => {
  test.beforeEach(async ({ page }) => openApp(page))

  test('newest layer is listed on top and painted on top', async ({ page }) => {
    await drawLine(page, [[0, 0], [6, 0]])
    await drawLine(page, [[3, -3], [3, 3]])
    const names = await page.locator('[data-testid=layers-panel] > .layer-wrap > .row .name').allTextContents()
    expect(names).toEqual(['Line 2', 'Line 1'])
    // paint order: last fragment in the DOM is the top one
    const lastId = await page.locator('[data-testid=doc-layer] > g').last().getAttribute('data-id')
    const topLeaf = await state(page, (s) => s.tree[0].children[0].id)
    expect(Number(lastId)).toBe(topLeaf)
  })

  test('hide and show a layer', async ({ page }) => {
    await drawLine(page, [[0, 0], [6, 0]])
    const row = await layerRow(page, 'Line 1')
    await row.getByTestId('toggle-visible').click()
    expect(await fragmentKinds(page)).toEqual([])
    await row.getByTestId('toggle-visible').click()
    expect(await fragmentKinds(page)).toEqual(['line'])
  })

  test('locked layers cannot be selected on the canvas or erased', async ({ page }) => {
    await drawLine(page, [[0, 0], [6, 0]])
    const row = await layerRow(page, 'Line 1')
    await row.hover()
    await row.getByTestId('toggle-lock').click()
    await page.keyboard.press('v')
    await clickCell(page, 1, 0)
    expect(await state(page, (s) => s.selection.length)).toBe(0)
    await page.keyboard.press('e')
    await clickCell(page, 3, 0)
    expect(await fragmentKinds(page)).toEqual(['line'])
  })

  test('drag a layer to reorder the stack', async ({ page }) => {
    await drawLine(page, [[0, 0], [6, 0]])
    await drawLine(page, [[3, -3], [3, 3]])
    const top = await layerRow(page, 'Line 2')
    const bottom = await layerRow(page, 'Line 1')
    const b = (await bottom.boundingBox())!
    await top.dragTo(bottom, { targetPosition: { x: 60, y: b.height - 3 } })
    const names = await page.locator('[data-testid=layers-panel] > .layer-wrap > .row .name').allTextContents()
    expect(names).toEqual(['Line 1', 'Line 2'])
  })

  test('rename with double click', async ({ page }) => {
    await drawLine(page, [[0, 0], [6, 0]])
    const row = await layerRow(page, 'Line 1')
    await row.locator('.name').dblclick()
    await page.locator('input.rename').fill('Red Line')
    await page.locator('input.rename').press('Enter')
    await layerRow(page, 'Red Line')
    await page.keyboard.press('Control+z')
    await layerRow(page, 'Line 1')
  })

  test('dragging a line onto another line group joins the two lines', async ({ page }) => {
    await drawLine(page, [[0, 0], [6, 0]])
    await drawLine(page, [[0, 4], [6, 4]])
    const target = await layerRow(page, 'Line 1')
    const source = await layerRow(page, 'Line 2')
    const t = (await target.boundingBox())!
    await source.dragTo(target, { targetPosition: { x: 80, y: t.height / 2 } })
    const tree = await state(page, (s) => s.tree.map((g: any) => ({ name: g.name, children: g.children.length })))
    expect(tree).toEqual([{ name: 'Line 1', children: 2 }])
    const ids = await state(page, (s) => s.fragments.map((f: any) => (window as any).__transit.node(f.id).content.line_id))
    expect(new Set(ids).size).toBe(1)
  })

  test('new empty group and moving a layer into it', async ({ page }) => {
    await drawLine(page, [[0, 0], [6, 0]])
    await page.getByTestId('new-group-btn').click()
    const group = await layerRow(page, 'Group 1')
    const line = await layerRow(page, 'Line 1')
    const g = (await group.boundingBox())!
    await line.dragTo(group, { targetPosition: { x: 80, y: g.height / 2 } })
    const nested = await state(page, (s) => s.tree[0].children.map((c: any) => c.name))
    expect(nested).toEqual(['Line 1'])
  })

  test('clicking an object on the canvas shows and highlights its layer', async ({ page }) => {
    await drawLine(page, [[0, 0], [6, 0]])
    await page.keyboard.press('s')
    await clickCell(page, 3, 0)
    await page.keyboard.press('v')
    await clickCell(page, -4, -4)
    // Close the line group: the station's row is hidden inside it.
    await (await layerRow(page, 'Line 1')).locator('.caret').click()
    await expect(page.locator('[data-testid=layers-panel] .row', { hasText: 'Station 1' })).toHaveCount(0)
    await clickCell(page, 3, 0)
    const id = await state(page, (s) => s.selection[0])
    const row = page.locator(`[data-testid=layers-panel] .row[data-layer-id="${id}"]`)
    await expect(row).toBeVisible()
    await expect(row).toHaveClass(/selected/)
    await expect(row).toBeInViewport()
  })
})
