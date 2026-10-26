// Form controls driven like a user would, through Testing Library: find them by
// their role or text, type and click with user-event, check what they emit.
import { describe, expect, it } from 'vitest'
import { fireEvent, render, screen } from '@testing-library/vue'
import userEvent from '@testing-library/user-event'
import NumberField from '../../src/components/ui/NumberField.vue'
import ColorField from '../../src/components/ui/ColorField.vue'
import Segmented from '../../src/components/ui/Segmented.vue'
import HelpTip from '../../src/components/ui/HelpTip.vue'
import { picking } from '../../src/core/eyedropper'

describe('NumberField', () => {
  function setup(props: Record<string, unknown> = {}) {
    const user = userEvent.setup()
    const r = render(NumberField, { props: { modelValue: 5, label: 'Width', min: 0, max: 100, ...props } })
    return { user, input: screen.getByRole('textbox'), ...r }
  }

  it('commits a typed value on Enter, clamped to its range', async () => {
    const { user, input, emitted } = setup()
    await user.clear(input)
    await user.type(input, '250{Enter}')
    expect(emitted('commit')).toEqual([[100]])
    expect(input).toHaveValue('100')
  })

  it('puts the old value back when the text is not a number or Escape is pressed', async () => {
    const { user, input, emitted } = setup()
    await user.clear(input)
    await user.type(input, 'abc')
    await user.tab()
    expect(input).toHaveValue('5')
    await user.clear(input)
    await user.type(input, '42{Escape}')
    expect(input).toHaveValue('5')
    expect(emitted('commit')).toBeUndefined()
  })

  it('steps with the arrow keys, ten at a time with Shift', async () => {
    const { user, input, emitted } = setup({ step: 0.5 })
    await user.click(input)
    await user.keyboard('{ArrowUp}')
    await user.keyboard('{Shift>}{ArrowDown}{/Shift}')
    expect(emitted('update:modelValue')).toEqual([[5.5], [0]])
  })

  it('rounds what it shows to its precision and follows the model value', async () => {
    const { input, rerender } = setup({ modelValue: 1.23456, precision: 2 })
    expect(input).toHaveValue('1.23')
    await rerender({ modelValue: 7 })
    expect(input).toHaveValue('7')
  })

  it('scrubs the value by dragging its label and commits once on release', async () => {
    const { emitted, rerender } = setup()
    const label = screen.getByText('Width')
    label.setPointerCapture = () => {}
    await fireEvent.pointerDown(label, { clientX: 100, pointerId: 1 })
    await fireEvent.pointerMove(label, { clientX: 112 })
    expect(emitted('update:modelValue')).toEqual([[9]])
    await rerender({ modelValue: 9 })
    await fireEvent.pointerUp(label)
    expect(emitted('commit')).toEqual([[9]])
  })

  it('shows its help next to the label', () => {
    setup({ help: 'How wide the line is.' })
    expect(screen.getByRole('img', { name: 'How wide the line is.' })).toBeInTheDocument()
  })
})

describe('ColorField', () => {
  function setup(modelValue = '#ff0000') {
    const user = userEvent.setup()
    const r = render(ColorField, { props: { modelValue, label: 'Color' } })
    return { user, hex: screen.getByRole('textbox'), ...r }
  }

  it('accepts hex colors with or without # and expands the short form', async () => {
    const { user, hex, emitted } = setup()
    await user.clear(hex)
    await user.type(hex, '0A0{Enter}')
    await user.clear(hex)
    await user.type(hex, '#123456')
    await user.tab()
    expect(emitted('update:modelValue')).toEqual([['#00aa00'], ['#123456']])
  })

  it('rejects anything else and keeps the current color', async () => {
    const { user, hex, emitted } = setup()
    await user.clear(hex)
    await user.type(hex, 'blue{Enter}')
    expect(hex).toHaveValue('#ff0000')
    expect(emitted('update:modelValue')).toBeUndefined()
  })

  it('turns the eyedropper on and off', async () => {
    const { user } = setup()
    const dropper = screen.getByTestId('eyedropper')
    await user.click(dropper)
    expect(picking.value).not.toBeNull()
    expect(dropper).toHaveClass('on')
    await user.click(dropper)
    expect(picking.value).toBeNull()
  })
})

describe('Segmented', () => {
  const options = [
    { value: 'start', label: 'Left' },
    { value: 'middle', label: 'Center' },
    { value: 'end', label: 'Right' },
  ]

  it('is a radio group that marks the current value and emits the clicked one', async () => {
    const user = userEvent.setup()
    const { emitted } = render(Segmented, { props: { modelValue: 'middle', options } })
    expect(screen.getByRole('radiogroup')).toBeInTheDocument()
    expect(screen.getByRole('radio', { name: 'Center' })).toHaveAttribute('aria-checked', 'true')
    expect(screen.getByRole('radio', { name: 'Left' })).toHaveAttribute('aria-checked', 'false')
    await user.click(screen.getByRole('radio', { name: 'Right' }))
    expect(emitted('update:modelValue')).toEqual([['end']])
  })
})

describe('HelpTip', () => {
  it('shows its text in a tooltip while hovered or focused', async () => {
    const user = userEvent.setup()
    render(HelpTip, { props: { text: 'Distance between dashes.' } })
    const tip = screen.getByRole('img', { name: 'Distance between dashes.' })
    expect(screen.queryByRole('tooltip')).toBeNull()
    await user.hover(tip)
    expect(screen.getByRole('tooltip')).toHaveTextContent('Distance between dashes.')
    await user.unhover(tip)
    expect(screen.queryByRole('tooltip')).toBeNull()
    await user.tab()
    expect(tip).toHaveFocus()
    expect(screen.getByRole('tooltip')).toBeInTheDocument()
  })
})
