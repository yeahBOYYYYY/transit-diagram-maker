// Browser file helpers for saving and opening documents.

export function downloadText(name: string, text: string, mime: string): void {
  const blob = new Blob([text], { type: mime })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = name
  a.style.display = 'none'
  document.body.appendChild(a)
  a.click()
  a.remove()
  setTimeout(() => URL.revokeObjectURL(url), 1000)
}

export function pickTextFile(accept: string): Promise<{ name: string; text: string } | null> {
  return new Promise((resolve) => {
    const input = document.createElement('input')
    input.type = 'file'
    input.accept = accept
    input.style.display = 'none'
    input.addEventListener('change', async () => {
      const file = input.files?.[0]
      input.remove()
      resolve(file ? { name: file.name, text: await file.text() } : null)
    })
    input.addEventListener('cancel', () => {
      input.remove()
      resolve(null)
    })
    document.body.appendChild(input)
    input.click()
  })
}

// File System Access API (Chromium): lets Save write back to the same file
// instead of downloading a new copy each time. Not in TypeScript's DOM lib yet.
export interface FileHandle {
  readonly name: string
  getFile(): Promise<File>
  createWritable(): Promise<{ write(data: string): Promise<void>; close(): Promise<void> }>
}

interface PickerOptions {
  suggestedName?: string
  types?: { description: string; accept: Record<string, string[]> }[]
}

interface FsWindow {
  showSaveFilePicker?: (o: PickerOptions) => Promise<FileHandle>
  showOpenFilePicker?: (o: PickerOptions) => Promise<FileHandle[]>
}

/** Saved documents are SVG pictures with the editor data embedded. */
export const TRANSIT_EXT = '.transit.svg'
const TRANSIT_TYPES = [{ description: 'Transit diagram (SVG)', accept: { 'image/svg+xml': ['.svg'] } }]

/** Document name for a file name: without `.transit.svg` (or `.svg`). */
export function docNameOf(fileName: string): string {
  return fileName.replace(/(\.transit)?\.svg$/i, '')
}

function fsWindow(): FsWindow {
  return window as unknown as FsWindow
}

export function canSaveInPlace(): boolean {
  return typeof fsWindow().showSaveFilePicker === 'function'
}

function isAbort(err: unknown): boolean {
  return err instanceof DOMException && err.name === 'AbortError'
}

/** Ask where to save; null when the user cancels. */
export async function pickSaveHandle(suggestedName: string): Promise<FileHandle | null> {
  try {
    return await fsWindow().showSaveFilePicker!({ suggestedName, types: TRANSIT_TYPES })
  } catch (err) {
    if (isAbort(err)) return null
    throw err
  }
}

export async function writeHandle(handle: FileHandle, text: string): Promise<void> {
  const w = await handle.createWritable()
  await w.write(text)
  await w.close()
}

/** Open a .transit.svg file, keeping its handle for saving back when the browser allows it. */
export async function pickTransitFile(): Promise<{ name: string; text: string; handle: FileHandle | null } | null> {
  const open = fsWindow().showOpenFilePicker
  if (typeof open !== 'function') {
    const f = await pickTextFile('.svg,image/svg+xml')
    return f && { ...f, handle: null }
  }
  try {
    const [handle] = await open({ types: TRANSIT_TYPES })
    const file = await handle.getFile()
    return { name: file.name, text: await file.text(), handle }
  } catch (err) {
    if (isAbort(err)) return null
    throw err
  }
}

/** Ask the user for one file. */
export function pickFile(accept: string): Promise<File | null> {
  return new Promise((resolve) => {
    const input = document.createElement('input')
    input.type = 'file'
    input.accept = accept
    input.style.display = 'none'
    input.addEventListener('change', () => {
      const file = input.files?.[0] ?? null
      input.remove()
      resolve(file)
    })
    input.addEventListener('cancel', () => {
      input.remove()
      resolve(null)
    })
    document.body.appendChild(input)
    input.click()
  })
}

const FONT_TYPES: Record<string, string> = { ttf: 'font/ttf', otf: 'font/otf', woff: 'font/woff', woff2: 'font/woff2' }

/** A font file as a base64 data URL with a font MIME type taken from its extension. */
export async function fontDataUrl(file: File): Promise<string> {
  const ext = file.name.split('.').pop()?.toLowerCase() ?? ''
  const mime = FONT_TYPES[ext] ?? 'application/octet-stream'
  const bytes = new Uint8Array(await file.arrayBuffer())
  let bin = ''
  for (let i = 0; i < bytes.length; i += 0x8000) bin += String.fromCharCode(...bytes.subarray(i, i + 0x8000))
  return `data:${mime};base64,${btoa(bin)}`
}

/** File name without its extension, as a default display name. */
export function baseName(name: string): string {
  return name.replace(/\.[^.]+$/, '').replace(/[_-]+/g, ' ').trim()
}
