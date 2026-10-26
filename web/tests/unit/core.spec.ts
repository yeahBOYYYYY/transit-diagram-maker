// Small core modules: file helpers, fonts, line styles, the action catalogue
// and the application information.
import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { afterEach, describe, expect, it, vi } from 'vitest'
import * as files from '../../src/core/files'
import { FONT_FAMILIES, fontChoices, fontLabel, uploadedFamily } from '../../src/core/fonts'
import { STROKE_KINDS, hasChannel, hasDashes } from '../../src/core/lineStyles'
import { ACTIONS, ACTION_IDS } from '../../src/core/actions'
import { APP } from '../../src/core/about'
import { Engine } from '../../src/core/engine'

type PickerWindow = { showSaveFilePicker?: unknown; showOpenFilePicker?: unknown }
const win = window as unknown as PickerWindow

afterEach(() => {
  delete win.showSaveFilePicker
  delete win.showOpenFilePicker
  vi.restoreAllMocks()
})

describe('file names', () => {
  it('drop the .transit.svg or .svg extension for the document name', () => {
    expect(files.docNameOf('Metro.transit.svg')).toBe('Metro')
    expect(files.docNameOf('Metro.SVG')).toBe('Metro')
    expect(files.docNameOf('notes.txt')).toBe('notes.txt')
  })

  it('turn an uploaded file name into a display name', () => {
    expect(files.baseName('city_sans-bold.woff2')).toBe('city sans bold')
    expect(files.baseName('star.svg')).toBe('star')
  })
})

describe('file helpers', () => {
  it('encode a font file as a data URL with the MIME type of its extension', async () => {
    const font = new File([new Uint8Array([0x77, 0x4f, 0x46, 0x32])], 'City.WOFF2')
    expect(await files.fontDataUrl(font)).toBe(`data:font/woff2;base64,${btoa('wOF2')}`)
    const other = new File([new Uint8Array([1])], 'font.bin')
    expect(await files.fontDataUrl(other)).toMatch(/^data:application\/octet-stream;base64,/)
  })

  it('download text through a temporary link', () => {
    vi.useFakeTimers()
    const create = vi.spyOn(URL, 'createObjectURL').mockReturnValue('blob:x')
    const revoke = vi.spyOn(URL, 'revokeObjectURL').mockImplementation(() => {})
    let clicked: HTMLAnchorElement | null = null
    vi.spyOn(HTMLAnchorElement.prototype, 'click').mockImplementation(function (this: HTMLAnchorElement) {
      clicked = this
    })
    files.downloadText('map.svg', '<svg/>', 'image/svg+xml')
    expect(create).toHaveBeenCalledWith(expect.any(Blob))
    expect(clicked!.download).toBe('map.svg')
    expect(clicked!.isConnected).toBe(false)
    vi.runAllTimers()
    expect(revoke).toHaveBeenCalledWith('blob:x')
    vi.useRealTimers()
  })

  it('save in place only where the browser has a save picker', async () => {
    expect(files.canSaveInPlace()).toBe(false)
    const handle = { name: 'a.transit.svg' }
    win.showSaveFilePicker = vi.fn().mockResolvedValue(handle)
    expect(files.canSaveInPlace()).toBe(true)
    expect(await files.pickSaveHandle('a.transit.svg')).toBe(handle)
    expect(win.showSaveFilePicker).toHaveBeenCalledWith(expect.objectContaining({ suggestedName: 'a.transit.svg' }))
  })

  it('treat a cancelled picker as no choice, and pass other errors on', async () => {
    win.showSaveFilePicker = vi.fn().mockRejectedValue(new DOMException('cancelled', 'AbortError'))
    expect(await files.pickSaveHandle('a')).toBeNull()
    win.showSaveFilePicker = vi.fn().mockRejectedValue(new Error('disk full'))
    await expect(files.pickSaveHandle('a')).rejects.toThrow('disk full')
    win.showOpenFilePicker = vi.fn().mockRejectedValue(new DOMException('cancelled', 'AbortError'))
    expect(await files.pickTransitFile()).toBeNull()
  })

  it('open a file through the picker and keep its handle', async () => {
    const file = new File(['<svg/>'], 'Map.transit.svg')
    const handle = { name: file.name, getFile: async () => file }
    win.showOpenFilePicker = vi.fn().mockResolvedValue([handle])
    expect(await files.pickTransitFile()).toEqual({ name: 'Map.transit.svg', text: '<svg/>', handle })
  })

  it('write the whole text to a handle and close it', async () => {
    const calls: string[] = []
    const handle = {
      name: 'x',
      getFile: async () => new File([], 'x'),
      createWritable: async () => ({ write: async (t: string) => void calls.push(`write ${t}`), close: async () => void calls.push('close') }),
    }
    await files.writeHandle(handle, 'data')
    expect(calls).toEqual(['write data', 'close'])
  })

  it('fall back to a file input, resolving to null when cancelled', async () => {
    vi.spyOn(HTMLInputElement.prototype, 'click').mockImplementation(function (this: HTMLInputElement) {
      this.dispatchEvent(new Event('cancel'))
    })
    expect(await files.pickTransitFile()).toBeNull()
    expect(document.querySelector('input[type=file]')).toBeNull()
  })
})

describe('fonts', () => {
  it('label a font by its first family and offer uploaded ones after the built-in ones', () => {
    expect(fontLabel('"Trebuchet MS", sans-serif')).toBe('Trebuchet MS')
    expect(uploadedFamily('City Sans')).toBe('"City Sans", sans-serif')
    expect(fontChoices(['City Sans'])).toEqual([...FONT_FAMILIES, '"City Sans", sans-serif'])
  })

  it('name uploaded fonts the way the engine does', () => {
    const e = new Engine()
    expect(e.addFont('City Sans', 'data:font/woff2;base64,d09GMgAB')).toBe(uploadedFamily('City Sans'))
  })
})

describe('line styles', () => {
  it('offer every stroke kind once, and tell which have a channel or dashes', () => {
    expect(new Set(STROKE_KINDS.map((k) => k.value)).size).toBe(STROKE_KINDS.length)
    expect(STROKE_KINDS.filter((k) => hasChannel(k.value)).map((k) => k.value)).toEqual(['hollow'])
    expect(STROKE_KINDS.filter((k) => hasDashes(k.value)).map((k) => k.value)).toEqual(['inner_dashed'])
  })

  it('are all accepted by the engine', () => {
    const e = new Engine()
    for (const { value } of STROKE_KINDS) {
      expect(() => e.createLine([{ x: 0, y: 0 }, { x: 3, y: 0 }], { kind: value })).not.toThrow()
    }
  })
})

describe('action catalogue', () => {
  it('gives every action a label and a category', () => {
    for (const id of ACTION_IDS) {
      expect(ACTIONS[id].label, id).not.toBe('')
      expect(ACTIONS[id].category, id).not.toBe('')
    }
  })

  it('spells words out in labels', () => {
    // The UI wording rule: no abbreviations like "Max", "px" or "Ctrl" in labels.
    for (const id of ACTION_IDS) {
      expect(ACTIONS[id].label, id).not.toMatch(/\b(px|ms|Max|Min|Ctrl|Alt|Btn|Img|Doc|Pref)\b/)
    }
  })
})

describe('application information', () => {
  it('shows the version shared by the web app and the Rust crates', () => {
    const cargo = readFileSync(resolve(__dirname, '../../../Cargo.toml'), 'utf8')
    const version = cargo.match(/\[workspace\.package\][^[]*?version\s*=\s*"([^"]+)"/)![1]
    expect(APP.version).toBe(version)
    expect(APP.repoUrl).toMatch(/^https:\/\/github\.com\//)
    expect(APP.issuesUrl.startsWith(APP.repoUrl)).toBe(true)
  })
})
