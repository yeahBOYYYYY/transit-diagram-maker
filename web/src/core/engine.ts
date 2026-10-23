// Typed wrapper around the WebAssembly editor. Everything structured crosses
// the boundary as JSON.
import { TransitEditor } from '../wasm/pkg/transit_wasm.js'
import type {
  Cell,
  CreatedLine,
  CreatedStation,
  DocNode,
  DocSettings,
  Fragment,
  Icon,
  IconInfo,
  Id,
  LineHit,
  LineStyle,
  Rect,
  RouteMode,
  StationKind,
  StationOptions,
  TextBox,
  TreeItem,
} from './types'

const j = JSON.stringify

export class Engine {
  private ed: TransitEditor

  constructor() {
    this.ed = new TransitEditor()
  }

  // files
  newDocument(): void {
    this.ed.newDocument()
  }
  loadTransit(text: string): void {
    this.ed.loadTransit(text)
  }
  saveTransit(): string {
    return this.ed.saveTransit()
  }
  exportSvg(): string {
    return this.ed.exportSvg()
  }

  // queries
  fragments(): Fragment[] {
    return JSON.parse(this.ed.fragments())
  }
  tree(): TreeItem[] {
    return JSON.parse(this.ed.tree())
  }
  settings(): DocSettings {
    return JSON.parse(this.ed.settings())
  }
  node(id: Id): DocNode | null {
    return JSON.parse(this.ed.node(id))
  }
  bounds(ids: Id[]): Rect | null {
    return JSON.parse(this.ed.bounds(j(ids)))
  }
  lineHitAt(cell: Cell): LineHit | null {
    return JSON.parse(this.ed.lineHitAt(cell.x, cell.y))
  }
  nearestLineCell(x: number, y: number, maxDist: number): LineHit | null {
    return JSON.parse(this.ed.nearestLineCell(x, y, maxDist))
  }
  previewLine(points: Cell[], style: Partial<LineStyle>): string {
    return this.ed.previewLine(j(points), j(style))
  }
  previewPill(cells: Cell[], outer: string, inner: string): string {
    return this.ed.previewPill(j(cells), outer, inner)
  }
  route(a: Cell, b: Cell, mode: RouteMode): Cell[] {
    return JSON.parse(this.ed.route(a.x, a.y, b.x, b.y, mode))
  }
  static iconLibrary(): IconInfo[] {
    return JSON.parse(TransitEditor.iconLibrary())
  }

  // history
  undo(): boolean {
    return this.ed.undo()
  }
  redo(): boolean {
    return this.ed.redo()
  }
  canUndo(): boolean {
    return this.ed.canUndo()
  }
  canRedo(): boolean {
    return this.ed.canRedo()
  }
  setHistoryLimit(n: number): void {
    this.ed.setHistoryLimit(Math.max(1, Math.floor(n)))
  }
  begin(): void {
    this.ed.begin()
  }
  rebase(): void {
    this.ed.rebase()
  }
  commit(): void {
    this.ed.commit()
  }
  cancel(): void {
    this.ed.cancel()
  }

  // mutations
  setSettings(s: DocSettings): void {
    this.ed.setSettings(j(s))
  }
  createLine(points: Cell[], style: Partial<LineStyle>): CreatedLine {
    return JSON.parse(this.ed.createLine(j(points), j(style)))
  }
  erase(cells: Cell[]): number {
    return this.ed.erase(j(cells))
  }
  addStation(kind: StationKind, cell: Cell, opts: Partial<StationOptions>): CreatedStation {
    return JSON.parse(this.ed.addStation(kind, cell.x, cell.y, j(opts)))
  }
  addPill(cells: Cell[], opts: Partial<StationOptions>): CreatedStation {
    return JSON.parse(this.ed.addPill(j(cells), j(opts)))
  }
  setStationKind(id: Id, kind: StationKind): void {
    this.ed.setStationKind(id, kind)
  }
  moveStationTo(id: Id, x: number, y: number): Cell {
    return JSON.parse(this.ed.moveStationTo(id, x, y))
  }
  addText(t: Partial<TextBox>): Id {
    return this.ed.addText(j(t))
  }
  addIcon(i: Partial<Icon>): Id {
    return this.ed.addIcon(j(i))
  }
  updateContent(id: Id, patch: Record<string, unknown>): void {
    this.ed.updateContent(id, j(patch))
  }
  rename(id: Id, name: string): void {
    this.ed.rename(id, name)
  }
  setVisible(id: Id, v: boolean): void {
    this.ed.setVisible(id, v)
  }
  setLocked(id: Id, v: boolean): void {
    this.ed.setLocked(id, v)
  }
  setExpanded(id: Id, v: boolean): void {
    this.ed.setExpanded(id, v)
  }
  delete(ids: Id[]): number {
    return this.ed.delete(j(ids))
  }
  group(ids: Id[]): Id {
    return this.ed.group(j(ids))
  }
  ungroup(id: Id): Id[] {
    return JSON.parse(this.ed.ungroup(id))
  }
  newGroup(): Id {
    return this.ed.newGroup()
  }
  /** `parent` null = root; `index` is bottom-to-top. */
  moveNode(id: Id, parent: Id | null, index: number): void {
    this.ed.moveNode(id, parent ?? -1, index)
  }
  restack(ids: Id[], delta: number): void {
    this.ed.restack(j(ids), delta)
  }
  duplicate(ids: Id[]): Id[] {
    return JSON.parse(this.ed.duplicate(j(ids)))
  }
  /** `free` lets stations leave their line (Alt-drag) instead of re-snapping. */
  translate(ids: Id[], dx: number, dy: number, free = false): void {
    this.ed.translate(j(ids), dx, dy, free)
  }
  rotate(ids: Id[], deg: number, pivot: { x: number; y: number }): number {
    return this.ed.rotate(j(ids), deg, pivot.x, pivot.y)
  }
}
