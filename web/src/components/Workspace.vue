<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch, computed } from 'vue';
import { engineBridge } from '../engineBridge';
import { EditorStateResponse, RenderPrimitive, DVec2, PathSegment } from '../types';

const props = defineProps<{
  state: EditorStateResponse | null;
}>();

const canvasRef = ref<HTMLCanvasElement | null>(null);
const containerRef = ref<HTMLDivElement | null>(null);

let isMiddlePanning = false;
let lastPointerPos = { x: 0, y: 0 };
const cursorCoords = ref<{ x: number; y: number }>({ x: 0, y: 0 });
const zoomPercent = ref<number>(100);

const activeToolCursor = computed(() => {
  switch (props.state?.active_tool) {
    case 'Pen': return 'crosshair';
    case 'Station': return 'pointer';
    case 'Erase': return 'crosshair';
    case 'Text': return 'text';
    default: return 'default';
  }
});

onMounted(() => {
  window.addEventListener('resize', handleResize);
  window.addEventListener('keydown', handleKeyDown);
  window.addEventListener('keyup', handleKeyUp);
  handleResize();
  renderScene();
});

onUnmounted(() => {
  window.removeEventListener('resize', handleResize);
  window.removeEventListener('keydown', handleKeyDown);
  window.removeEventListener('keyup', handleKeyUp);
});

watch(() => props.state, () => {
  renderScene();
  if (props.state?.camera) {
    zoomPercent.value = Math.round(props.state.camera.zoom * 100);
  }
}, { deep: true });

function handleResize() {
  if (!containerRef.value || !canvasRef.value) return;
  const rect = containerRef.value.getBoundingClientRect();
  const dpr = window.devicePixelRatio || 1;

  canvasRef.value.width = rect.width * dpr;
  canvasRef.value.height = rect.height * dpr;
  canvasRef.value.style.width = `${rect.width}px`;
  canvasRef.value.style.height = `${rect.height}px`;

  engineBridge.dispatch({
    type: 'ResizeViewport',
    payload: { width: rect.width, height: rect.height },
  });
}

function handlePointerDown(e: PointerEvent) {
  if (!canvasRef.value) return;
  canvasRef.value.setPointerCapture(e.pointerId);

  const rect = canvasRef.value.getBoundingClientRect();
  const screenPos: DVec2 = { x: e.clientX - rect.left, y: e.clientY - rect.top };

  lastPointerPos = { x: e.clientX, y: e.clientY };

  if (e.button === 1 || e.altKey) {
    isMiddlePanning = true;
    return;
  }

  engineBridge.dispatch({
    type: 'PointerDown',
    payload: {
      screen_pos: screenPos,
      shift: e.shiftKey,
      alt: e.altKey,
      ctrl: e.ctrlKey || e.metaKey,
      button: e.button,
    },
  });
}

function handlePointerMove(e: PointerEvent) {
  if (!canvasRef.value) return;
  const rect = canvasRef.value.getBoundingClientRect();
  const screenPos: DVec2 = { x: e.clientX - rect.left, y: e.clientY - rect.top };

  if (props.state?.camera) {
    const cam = props.state.camera;
    const cx = cam.width * 0.5;
    const cy = cam.height * 0.5;
    const wx = Math.round(cam.pan.x + (screenPos.x - cx) / cam.zoom);
    const wy = Math.round(cam.pan.y + (screenPos.y - cy) / cam.zoom);
    cursorCoords.value = { x: wx, y: wy };
  }

  if (isMiddlePanning) {
    const dx = e.clientX - lastPointerPos.x;
    const dy = e.clientY - lastPointerPos.y;
    lastPointerPos = { x: e.clientX, y: e.clientY };

    if (props.state?.camera) {
      const cam = props.state.camera;
      cam.pan.x -= dx / cam.zoom;
      cam.pan.y -= dy / cam.zoom;
      renderScene();
    }
    return;
  }

  engineBridge.dispatch({
    type: 'PointerMove',
    payload: {
      screen_pos: screenPos,
      shift: e.shiftKey,
      alt: e.altKey,
      ctrl: e.ctrlKey || e.metaKey,
    },
  });
}

function handlePointerUp(e: PointerEvent) {
  if (!canvasRef.value) return;
  try {
    canvasRef.value.releasePointerCapture(e.pointerId);
  } catch (_) {}

  const rect = canvasRef.value.getBoundingClientRect();
  const screenPos: DVec2 = { x: e.clientX - rect.left, y: e.clientY - rect.top };

  if (isMiddlePanning) {
    isMiddlePanning = false;
    return;
  }

  engineBridge.dispatch({
    type: 'PointerUp',
    payload: {
      screen_pos: screenPos,
      shift: e.shiftKey,
      alt: e.altKey,
      ctrl: e.ctrlKey || e.metaKey,
      button: e.button,
    },
  });
}

function handleWheel(e: WheelEvent) {
  e.preventDefault();
  if (!canvasRef.value) return;
  const rect = canvasRef.value.getBoundingClientRect();
  const screenPos: DVec2 = { x: e.clientX - rect.left, y: e.clientY - rect.top };

  engineBridge.dispatch({
    type: 'Wheel',
    payload: {
      screen_pos: screenPos,
      delta_y: e.deltaY,
    },
  });
}

function handleKeyDown(e: KeyboardEvent) {
  if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) {
    return;
  }
  engineBridge.dispatch({
    type: 'KeyDown',
    payload: {
      key: e.key,
      shift: e.shiftKey,
      ctrl: e.ctrlKey || e.metaKey,
    },
  });
}

function handleKeyUp(e: KeyboardEvent) {
  engineBridge.dispatch({
    type: 'KeyUp',
    payload: {
      key: e.key,
      shift: e.shiftKey,
      ctrl: e.ctrlKey || e.metaKey,
    },
  });
}

// =========================================================================
// Canvas Vector & Grid Renderer
// =========================================================================

function renderScene() {
  const canvas = canvasRef.value;
  if (!canvas || !props.state) return;
  const ctx = canvas.getContext('2d');
  if (!ctx) return;

  const dpr = window.devicePixelRatio || 1;
  const { camera, primitives, selected_node_ids } = props.state;
  const w = canvas.width / dpr;
  const h = canvas.height / dpr;

  ctx.save();
  ctx.scale(dpr, dpr);

  ctx.fillStyle = '#151518';
  ctx.fillRect(0, 0, w, h);

  ctx.translate(w * 0.5, h * 0.5);
  ctx.scale(camera.zoom, camera.zoom);
  ctx.translate(-camera.pan.x, -camera.pan.y);

  drawProceduralGrid(ctx, camera, w, h);

  for (const prim of primitives) {
    drawPrimitive(ctx, prim, selected_node_ids.includes(prim.id));
  }

  ctx.restore();
}

function drawProceduralGrid(ctx: CanvasRenderingContext2D, cam: any, viewportW: number, viewportH: number) {
  const spacing = cam.grid_spacing;
  const zoom = cam.zoom;
  const screenUnit = spacing * zoom;

  const left = cam.pan.x - (viewportW * 0.5) / zoom;
  const right = cam.pan.x + (viewportW * 0.5) / zoom;
  const top = cam.pan.y - (viewportH * 0.5) / zoom;
  const bottom = cam.pan.y + (viewportH * 0.5) / zoom;

  const startX = Math.floor(left / spacing) * spacing;
  const endX = Math.ceil(right / spacing) * spacing;
  const startY = Math.floor(top / spacing) * spacing;
  const endY = Math.ceil(bottom / spacing) * spacing;

  const minorOpacity = screenUnit < 12 ? 0 : screenUnit < 24 ? (screenUnit - 12) / 12 : 1;

  ctx.lineWidth = 1 / zoom;

  if (minorOpacity > 0.02) {
    ctx.strokeStyle = `rgba(255, 255, 255, ${0.06 * minorOpacity})`;
    ctx.beginPath();
    for (let x = startX; x <= endX; x += spacing) {
      if (Math.round(x / spacing) % 5 !== 0) {
        ctx.moveTo(x, top);
        ctx.lineTo(x, bottom);
      }
    }
    for (let y = startY; y <= endY; y += spacing) {
      if (Math.round(y / spacing) % 5 !== 0) {
        ctx.moveTo(left, y);
        ctx.lineTo(right, y);
      }
    }
    ctx.stroke();
  }

  const majorSpacing = spacing * 5;
  const majorStartX = Math.floor(left / majorSpacing) * majorSpacing;
  const majorEndX = Math.ceil(right / majorSpacing) * majorSpacing;
  const majorStartY = Math.floor(top / majorSpacing) * majorSpacing;
  const majorEndY = Math.ceil(bottom / majorSpacing) * majorSpacing;

  ctx.strokeStyle = 'rgba(255, 255, 255, 0.15)';
  ctx.beginPath();
  for (let x = majorStartX; x <= majorEndX; x += majorSpacing) {
    ctx.moveTo(x, top);
    ctx.lineTo(x, bottom);
  }
  for (let y = majorStartY; y <= majorEndY; y += majorSpacing) {
    ctx.moveTo(left, y);
    ctx.lineTo(right, y);
  }
  ctx.stroke();

  // Origin Crosshair
  ctx.strokeStyle = 'rgba(0, 122, 255, 0.5)';
  ctx.beginPath();
  ctx.moveTo(0, -15);
  ctx.lineTo(0, 15);
  ctx.moveTo(-15, 0);
  ctx.lineTo(15, 0);
  ctx.stroke();
}

function drawPrimitive(ctx: CanvasRenderingContext2D, prim: RenderPrimitive, isSelected: boolean) {
  const t = prim.primitive_type;

  if ('Path' in t) {
    const path = t.Path;
    ctx.beginPath();
    for (const seg of path.segments) {
      if (typeof seg === 'object') {
        if ('MoveTo' in seg) ctx.moveTo(seg.MoveTo.x, seg.MoveTo.y);
        else if ('LineTo' in seg) ctx.lineTo(seg.LineTo.x, seg.LineTo.y);
        else if ('QuadTo' in seg) ctx.quadraticCurveTo(seg.QuadTo.ctrl.x, seg.QuadTo.ctrl.y, seg.QuadTo.to.x, seg.QuadTo.to.y);
      } else if (seg === 'Close') {
        ctx.closePath();
      }
    }

    ctx.lineCap = path.line_cap.toLowerCase() as CanvasLineCap;
    ctx.lineJoin = 'round';

    if (path.stroke_width > 0) {
      ctx.strokeStyle = path.stroke_color;
      ctx.lineWidth = path.stroke_width;

      if (typeof path.stroke_style === 'object' && 'Dashed' in path.stroke_style) {
        ctx.setLineDash([path.stroke_style.Dashed.dash_len, path.stroke_style.Dashed.gap_len]);
      } else {
        ctx.setLineDash([]);
      }

      ctx.stroke();
      ctx.setLineDash([]);

      // Hollow pattern with inner color
      if (typeof path.stroke_style === 'object' && 'Hollow' in path.stroke_style) {
        const hollow = path.stroke_style.Hollow;
        ctx.strokeStyle = hollow.inner_color;
        ctx.lineWidth = hollow.inner_width;
        ctx.stroke();
      }
    }

    if (isSelected) {
      ctx.strokeStyle = '#007aff';
      ctx.lineWidth = Math.max(2, path.stroke_width + 4);
      ctx.globalAlpha = 0.35;
      ctx.stroke();
      ctx.globalAlpha = 1.0;
    }
  } else if ('Station' in t) {
    const st = t.Station;

    if (st.style === 'Dot') {
      if (st.span_cells === 0 || (st.center.x === st.end_pos.x && st.center.y === st.end_pos.y)) {
        // Standard single dot
        ctx.beginPath();
        ctx.arc(st.center.x, st.center.y, st.radius, 0, Math.PI * 2);
        ctx.fillStyle = st.fill_color;
        ctx.fill();
        ctx.strokeStyle = st.stroke_color;
        ctx.lineWidth = st.stroke_width;
        ctx.stroke();

        if (isSelected) {
          ctx.beginPath();
          ctx.arc(st.center.x, st.center.y, st.radius + 4, 0, Math.PI * 2);
          ctx.strokeStyle = '#007aff';
          ctx.lineWidth = 2;
          ctx.stroke();
        }
      } else {
        // Stretched slinky dot: rectangular body with semi-circles on the first and last grid cell
        const angle = Math.atan2(st.end_pos.y - st.center.y, st.end_pos.x - st.center.x);

        ctx.beginPath();
        // Semicircle at end_pos
        ctx.arc(st.end_pos.x, st.end_pos.y, st.radius, angle - Math.PI / 2, angle + Math.PI / 2, false);
        // Semicircle at center
        ctx.arc(st.center.x, st.center.y, st.radius, angle + Math.PI / 2, angle - Math.PI / 2, false);
        ctx.closePath();

        ctx.fillStyle = st.fill_color;
        ctx.fill();
        ctx.strokeStyle = st.stroke_color;
        ctx.lineWidth = st.stroke_width;
        ctx.stroke();

        if (isSelected) {
          ctx.beginPath();
          ctx.arc(st.end_pos.x, st.end_pos.y, st.radius + 4, angle - Math.PI / 2, angle + Math.PI / 2, false);
          ctx.arc(st.center.x, st.center.y, st.radius + 4, angle + Math.PI / 2, angle - Math.PI / 2, false);
          ctx.closePath();
          ctx.strokeStyle = '#007aff';
          ctx.lineWidth = 2;
          ctx.stroke();
        }
      }
    } else if (st.style === 'Tick') {
      ctx.save();
      ctx.translate(st.center.x, st.center.y);
      ctx.rotate(st.rotation_rad);
      ctx.beginPath();
      ctx.moveTo(-st.radius, 0);
      ctx.lineTo(st.radius, 0);
      ctx.strokeStyle = st.stroke_color;
      ctx.lineWidth = st.stroke_width;
      ctx.lineCap = 'round';
      ctx.stroke();

      if (isSelected) {
        ctx.beginPath();
        ctx.arc(0, 0, st.radius + 4, 0, Math.PI * 2);
        ctx.strokeStyle = '#007aff';
        ctx.lineWidth = 2;
        ctx.stroke();
      }
      ctx.restore();
    }
  } else if ('Text' in t) {
    const txt = t.Text;
    ctx.font = `600 ${txt.font_size}px -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif`;
    ctx.textBaseline = 'top';

    if (txt.is_colliding) {
      const bounds = txt.bounds;
      ctx.strokeStyle = '#ff3b30';
      ctx.lineWidth = 1.5;
      ctx.fillStyle = 'rgba(255, 59, 48, 0.15)';
      ctx.strokeRect(bounds.min.x - 4, bounds.min.y - 2, bounds.max.x - bounds.min.x + 8, bounds.max.y - bounds.min.y + 4);
      ctx.fillRect(bounds.min.x - 4, bounds.min.y - 2, bounds.max.x - bounds.min.x + 8, bounds.max.y - bounds.min.y + 4);

      ctx.fillStyle = '#ff453a';
    } else {
      ctx.fillStyle = txt.color;
    }

    ctx.fillText(txt.text, txt.pos.x, txt.pos.y);

    if (isSelected) {
      const bounds = txt.bounds;
      ctx.strokeStyle = '#007aff';
      ctx.lineWidth = 1;
      ctx.setLineDash([3, 3]);
      ctx.strokeRect(bounds.min.x - 2, bounds.min.y - 2, bounds.max.x - bounds.min.x + 4, bounds.max.y - bounds.min.y + 4);
      ctx.setLineDash([]);
    }
  }
}
</script>

<template>
  <div class="workspace-viewport" ref="containerRef" :style="{ cursor: activeToolCursor }">
    <canvas
      ref="canvasRef"
      @pointerdown="handlePointerDown"
      @pointermove="handlePointerMove"
      @pointerup="handlePointerUp"
      @wheel="handleWheel"
    ></canvas>

    <!-- Coordinate & Status HUD -->
    <div class="viewport-hud">
      <div class="hud-item">
        <span class="hud-label">ZOOM</span>
        <span class="hud-val">{{ zoomPercent }}%</span>
      </div>
      <div class="hud-divider"></div>
      <div class="hud-item">
        <span class="hud-label">GRID POS</span>
        <span class="hud-val">X: {{ cursorCoords.x }}</span>
        <span class="hud-val">Y: {{ cursorCoords.y }}</span>
      </div>
      <div class="hud-divider"></div>
      <div class="hud-item">
        <span class="hud-label">GRID LOCK</span>
        <span class="hud-val active">STRICT CHANNEL LOCK</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.workspace-viewport {
  flex: 1;
  position: relative;
  background-color: var(--bg-workspace);
  overflow: hidden;
}

canvas {
  width: 100%;
  height: 100%;
  display: block;
}

.viewport-hud {
  position: absolute;
  bottom: 14px;
  left: 14px;
  display: flex;
  align-items: center;
  gap: 10px;
  background: rgba(22, 22, 26, 0.88);
  backdrop-filter: blur(8px);
  border: 1px solid var(--border-color);
  padding: 5px 12px;
  border-radius: 6px;
  font-family: var(--font-mono);
  font-size: 11px;
  color: var(--text-muted);
  pointer-events: none;
}

.hud-item {
  display: flex;
  align-items: center;
  gap: 6px;
}

.hud-label {
  color: var(--text-dim);
  font-size: 9px;
  font-weight: 600;
}

.hud-val {
  color: var(--text-main);
}

.hud-val.active {
  color: #30d158;
}

.hud-divider {
  width: 1px;
  height: 12px;
  background-color: var(--border-color);
}
</style>
