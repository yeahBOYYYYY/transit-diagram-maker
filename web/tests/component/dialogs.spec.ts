// Dialogs and pickers rendered with Testing Library against a real engine.
import { describe, expect, it } from 'vitest'
import { render, screen, within } from '@testing-library/vue'
import userEvent from '@testing-library/user-event'
import { freshStore } from '../unit/helpers'
import ModalDialog from '../../src/components/dialogs/ModalDialog.vue'
import ReferenceDialog from '../../src/components/dialogs/ReferenceDialog.vue'
import AssetsDialog from '../../src/components/dialogs/AssetsDialog.vue'
import FontSelect from '../../src/components/ui/FontSelect.vue'
import { ACTIONS } from '../../src/core/actions'
import { FONT_FAMILIES, fontLabel } from '../../src/core/fonts'
import { prefs } from '../../src/stores/prefs'

describe('ModalDialog', () => {
  function setup(props: Record<string, unknown> = {}) {
    const user = userEvent.setup()
    const r = render(ModalDialog, { props: { title: 'Rename', ...props }, slots: { default: '<input aria-label="Name"/><textarea aria-label="Notes"></textarea>' } })
    return { user, dialog: screen.getByRole('dialog', { name: 'Rename' }), ...r }
  }

  it('confirms with OK or Enter and cancels with Cancel, Escape or the close button', async () => {
    const { user, emitted } = setup()
    await user.click(screen.getByRole('button', { name: 'OK' }))
    await user.click(screen.getByLabelText('Name'))
    await user.keyboard('{Enter}')
    expect(emitted('ok')).toHaveLength(2)
    await user.click(screen.getByRole('button', { name: 'Cancel' }))
    await user.keyboard('{Escape}')
    await user.click(screen.getByTitle('Close'))
    expect(emitted('cancel')).toHaveLength(3)
  })

  it('lets Enter start a new line in a text area', async () => {
    const { user, emitted } = setup()
    await user.click(screen.getByLabelText('Notes'))
    await user.keyboard('{Enter}')
    expect(emitted('ok')).toBeUndefined()
  })

  it('without a Cancel button, closing it means OK', async () => {
    const { user, emitted } = setup({ cancel: false, okLabel: 'Close' })
    expect(screen.queryByRole('button', { name: 'Cancel' })).toBeNull()
    await user.keyboard('{Escape}')
    await user.click(screen.getByRole('button', { name: 'Close' }))
    expect(emitted('ok')).toHaveLength(2)
    expect(emitted('cancel')).toBeUndefined()
  })
})

describe('ReferenceDialog', () => {
  it('lists every action that has a shortcut, with the current binding', () => {
    freshStore()
    prefs.shortcuts['tool.pen'] = 'B'
    prefs.shortcuts['edit.duplicate'] = ''
    render(ReferenceDialog)
    const row = (label: string) => screen.queryAllByRole('row').find((r) => within(r).queryByText(label, { exact: true }))
    expect(within(row(ACTIONS['tool.pen'].label)!).getByText('B')).toBeInTheDocument()
    expect(within(row(ACTIONS['edit.undo'].label)!).getByText('Ctrl+Z')).toBeInTheDocument()
    expect(row(ACTIONS['edit.duplicate'].label)).toBeUndefined()
  })
})

describe('AssetsDialog', () => {
  it('lists uploaded fonts and icons and removes them', async () => {
    const user = userEvent.setup()
    const store = freshStore()
    render(AssetsDialog)
    expect(screen.getByText('No fonts uploaded.')).toBeInTheDocument()
    expect(screen.getByText('No icons uploaded.')).toBeInTheDocument()
    store.run((e) => e.addFont('City Sans', 'data:font/woff2;base64,d09GMgAB'))
    store.run((e) => e.addIconAsset('Star', '<svg viewBox="0 0 10 10"><path d="M0 0H10V10Z"/></svg>'))
    const fonts = within(await screen.findByTestId('asset-fonts'))
    expect(await fonts.findByText('City Sans')).toBeInTheDocument()
    expect(within(screen.getByTestId('asset-icons')).getByText('Star')).toBeInTheDocument()
    await user.click(screen.getByTitle('Remove City Sans'))
    expect(await screen.findByText('No fonts uploaded.')).toBeInTheDocument()
    await user.click(screen.getByTitle('Remove Star'))
    expect(await screen.findByText('No icons uploaded.')).toBeInTheDocument()
    expect(store.state.assets).toEqual({ fonts: [], icons: [] })
  })
})

describe('FontSelect', () => {
  it('offers the built-in and uploaded fonts and emits the chosen one', async () => {
    const user = userEvent.setup()
    const store = freshStore()
    store.run((e) => e.addFont('City Sans', 'data:font/woff2;base64,d09GMgAB'))
    const { emitted } = render(FontSelect, { props: { modelValue: FONT_FAMILIES[0] } })
    const select = screen.getByRole('combobox')
    const labels = within(select).getAllByRole('option').map((o) => o.textContent)
    expect(labels).toEqual([...FONT_FAMILIES.map(fontLabel), 'City Sans', 'Upload font…'])
    await user.selectOptions(select, 'Georgia')
    expect(emitted('update:modelValue')).toEqual([['Georgia, serif']])
  })

  it('keeps a font that is no longer offered selectable', () => {
    freshStore()
    render(FontSelect, { props: { modelValue: '"Removed Font", sans-serif' } })
    expect(within(screen.getByRole('combobox')).getAllByRole('option')[0]).toHaveTextContent('Removed Font')
    expect(screen.getByRole('combobox')).toHaveDisplayValue('Removed Font')
  })
})
