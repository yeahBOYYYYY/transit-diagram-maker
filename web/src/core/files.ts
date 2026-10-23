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

const TRANSIT_TYPES = [{ description: 'Transit diagram', accept: { 'application/json': ['.transit'] } }]

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

/** Open a .transit file, keeping its handle for saving back when the browser allows it. */
export async function pickTransitFile(): Promise<{ name: string; text: string; handle: FileHandle | null } | null> {
  const open = fsWindow().showOpenFilePicker
  if (typeof open !== 'function') {
    const f = await pickTextFile('.transit,application/json')
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
