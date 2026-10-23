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
})
