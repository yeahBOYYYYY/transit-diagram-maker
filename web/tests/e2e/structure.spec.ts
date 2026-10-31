// What the app shows, checked as structure rather than pixels: accessibility
// snapshots for the workspace, menus and dialogs, and the canvas and export
// markup compared with the engine's. Nothing here depends on the fonts or the
// graphics of the machine running it. The drawing itself is pinned down by the
// Rust snapshot tests (crates/transit-core/tests/render_snapshots.rs).
import { expect, test } from '@playwright/test'
import { buildSampleMap, openApp, state } from './helpers'

test.describe('structure', () => {
  test.beforeEach(async ({ page }) => openApp(page))

  test('workspace: menu bar, tools, canvas, panels and status bar', async ({ page }) => {
    await expect(page.getByRole('navigation')).toMatchAriaSnapshot(`
      - img "Transit Diagram Maker"
      - button "Save (Ctrl+S)"
      - button "File"
      - button "Edit"
      - button "Layer"
      - button "Select"
      - button "View"
      - button "Window"
      - button "Help"
      - text: Untitled-1.transit.svg — Transit Diagram Maker
    `)
    await expect(page.getByTestId('tools-panel')).toMatchAriaSnapshot(`
      - button "Selection Tool (V)"
      - button "Hand Tool (H)"
      - button "Line Pen Tool (P)"
      - button "Erase Tool (E)"
      - button "Station Tool (S)"
      - button "Text Tool (T)"
      - button "Shape Tool (U)"
      - textbox: "#e03c31"
      - textbox: "#ffffff"
      - button "Swap Line and Inner Colors (X)"
      - button "Default Line and Inner Colors (D)"
    `)
    await expect(page.getByRole('main')).toBeVisible()
    await expect(page.getByTestId('layers-panel')).toMatchAriaSnapshot(`
      - text: No layers yet. Pick the Line Pen (P) and click on the canvas to start drawing.
    `)
    await expect(page.getByRole('button', { name: 'New group' })).toBeEnabled()
    await expect(page.getByRole('button', { name: 'Delete', exact: true })).toBeDisabled()
    await expect(page.getByRole('contentinfo')).toMatchAriaSnapshot(`
      - textbox "%": "100"
      - link "Transit Diagram Maker on GitHub"
    `)
  })

  test('workspace panels fill the window without overlapping', async ({ page }) => {
    const box = async (id: string) => (await page.getByTestId(id).boundingBox())!
    const [tools, canvas, props, layers, status, options] = await Promise.all(
      ['tools-panel', 'canvas', 'properties-panel', 'layers-panel', 'status-bar', 'options-bar'].map(box),
    )
    const viewport = page.viewportSize()!
    expect(tools.x + tools.width).toBeLessThanOrEqual(canvas.x + 1)
    expect(canvas.x + canvas.width).toBeLessThanOrEqual(props.x + 1)
    expect(props.y + props.height).toBeLessThanOrEqual(layers.y + 1)
    expect(options.y + options.height).toBeLessThanOrEqual(canvas.y + 1)
    expect(canvas.y + canvas.height).toBeLessThanOrEqual(status.y + 1)
    expect(status.y + status.height).toBeLessThanOrEqual(viewport.height + 1)
    // The canvas takes most of the window.
    expect(canvas.width * canvas.height).toBeGreaterThan(0.4 * viewport.width * viewport.height)
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth)).toBe(true)
  })

  test('File menu holds the file commands and the document settings', async ({ page }) => {
    await page.click('[data-menu=File]')
    await expect(page.getByRole('menu')).toMatchAriaSnapshot(`
      - menu:
        - menuitem "New Ctrl+Alt+N"
        - menuitem "Open… Ctrl+O"
        - menuitem "Save Ctrl+S"
        - menuitem "Save As… Ctrl+Shift+S"
        - menuitem "Export as SVG… Ctrl+Shift+E"
        - menuitem "Document Settings… Ctrl+Alt+D"
        - menuitem "Fonts & Icons…"
    `)
  })

  test('Layer menu lists its commands with their shortcuts', async ({ page }) => {
    await page.click('[data-menu=Layer]')
    await expect(page.getByRole('menu')).toMatchAriaSnapshot(`
      - menu:
        - menuitem "New Group Ctrl+Shift+N"
        - menuitem "Group Layers Ctrl+G" [disabled]
        - menuitem "Ungroup Layers Ctrl+Shift+G" [disabled]
        - menuitem "Hide / Show Ctrl+," [disabled]
        - menuitem "Lock / Unlock Ctrl+/" [disabled]
        - menuitem "Bring to Front Ctrl+Shift+]" [disabled]
        - menuitem "Bring Forward Ctrl+]" [disabled]
        - menuitem "Send Backward Ctrl+[" [disabled]
        - menuitem "Send to Back Ctrl+Shift+[" [disabled]
    `)
  })

  test('Document Settings dialog groups its settings', async ({ page }) => {
    await page.keyboard.press('Control+Alt+d')
    await expect(page.getByRole('dialog')).toMatchAriaSnapshot(`
      - dialog "Document Settings":
        - banner:
          - text: Document Settings
        - group "Drawing":
          - combobox /Pen routing/
        - group "Lines":
          - textbox /^Width/: "1"
          - textbox /^Curve radius/: "1.5"
          - textbox /^Round cap roundness/: "1"
        - group "Stations":
          - textbox /^Dot \\/ pill outer radius/: "0.62"
          - textbox /^Dot \\/ pill inner radius/: "0.4"
          - textbox /^New tick width/: "0.35"
          - textbox /^New tick height/: "0.55"
        - group "Station labels":
          - textbox /^Distance/: "1.3"
          - textbox /^Font size/: "0.75"
          - combobox /^Font/
        - group "Pictograms":
          - checkbox /Default size matches dot stations/ [checked]
    `)
  })

  test('the canvas draws exactly the engine fragments, in paint order', async ({ page }) => {
    await buildSampleMap(page)
    const expected: { id: number; kind: string; svg: string }[] = await state(page, (s) => s.fragments.map((f: any) => ({ id: f.id, kind: f.kind, svg: f.svg })))
    const count = (kind: string) => expected.filter((f) => f.kind === kind).length
    expect([count('line'), count('station'), count('text'), count('icon')]).toEqual([3, 6, 7, 1])
    const drawn = await page.getByTestId('doc-layer').evaluate((layer, fragments) => {
      // Serialize the expected markup the same way the browser serializes the canvas.
      const normalize = (markup: string) => {
        const g = document.createElementNS('http://www.w3.org/2000/svg', 'g')
        g.innerHTML = markup
        return g.innerHTML
      }
      const groups = [...layer.querySelectorAll(':scope > g[data-id]')]
      return {
        ids: groups.map((g) => Number(g.getAttribute('data-id'))),
        mismatched: fragments.filter((f) => normalize(f.svg) !== layer.querySelector(`:scope > g[data-id="${f.id}"]`)?.innerHTML).map((f) => f.id),
      }
    }, expected)
    expect(drawn.ids).toEqual(expected.map((f) => f.id))
    expect(drawn.mismatched).toEqual([])
  })

  test('the exported SVG is a valid picture with every visible shape', async ({ page }) => {
    await buildSampleMap(page)
    const { svg, count } = await state(page, (s) => ({ svg: (window as any).__transit.engine.value.exportSvg() as string, count: s.fragments.length }))
    await page.setContent(`<body style="margin:0">${svg.replace(/^<\?xml[^>]*>/, '')}</body>`)
    const picture = page.locator('body > svg')
    await expect(picture).toBeVisible()
    const shapes = await picture.evaluate((el: SVGSVGElement) => {
      const layers = [...el.querySelectorAll('g[id^="layer-"]')] as SVGGElement[]
      return { layers: layers.length, empty: layers.filter((g) => { const b = g.getBBox(); return !(b.width > 0 || b.height > 0) }).map((g) => g.id) }
    })
    expect(shapes.layers).toBe(count)
    expect(shapes.empty).toEqual([])
  })
})
