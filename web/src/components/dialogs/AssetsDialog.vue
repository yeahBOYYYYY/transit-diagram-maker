<script setup lang="ts">
// Fonts and SVG icons uploaded into the document. They are saved inside the
// .transit file and embedded in SVG exports.
import ModalDialog from './ModalDialog.vue'
import HelpTip from '../ui/HelpTip.vue'
import { uploadedFamily } from '../../core/fonts'
import { useEditor } from '../../stores/editor'
import { uploadFont, uploadIcon } from '../../stores/commands'

const store = useEditor()
</script>

<template>
  <ModalDialog title="Fonts & Icons" :width="520" ok-label="Close" :cancel="false" @ok="store.state.dialog = null">
    <p class="dim intro">Uploaded fonts and icons are saved inside this document, so the file and its SVG export stay self-contained.</p>
    <section data-testid="asset-fonts">
      <header>
        <h3>Fonts<HelpTip text="TrueType, OpenType or WOFF files. Uploaded fonts appear in every font list and are embedded in SVG exports of text that uses them." /></h3>
        <button class="ps-btn" data-testid="add-font" @click="uploadFont(store)">Upload font…</button>
      </header>
      <div v-if="!store.state.assets.fonts.length" class="dim empty">No fonts uploaded.</div>
      <div v-for="f in store.state.assets.fonts" :key="f" class="item">
        <span class="sample" :style="{ fontFamily: uploadedFamily(f) }">Aa</span>
        <span class="name">{{ f }}</span>
        <button class="ps-btn" :title="`Remove ${f}`" @click="store.run((e) => e.removeFont(f))">Remove</button>
      </div>
    </section>
    <section data-testid="asset-icons">
      <header>
        <h3>Icons<HelpTip text="SVG drawings for the pictogram tool. Scripts, links and styles are removed; shapes keep their colors, and parts drawn with currentColor take the pictogram's glyph color." /></h3>
        <button class="ps-btn" data-testid="add-icon" @click="uploadIcon(store)">Upload SVG…</button>
      </header>
      <div v-if="!store.state.assets.icons.length" class="dim empty">No icons uploaded.</div>
      <div v-for="i in store.state.assets.icons" :key="i.name" class="item">
        <svg viewBox="0 0 24 24" width="28" height="28" v-html="i.svg" />
        <span class="name">{{ i.label }}</span>
        <button class="ps-btn" :title="`Remove ${i.label}`" @click="store.run((e) => e.removeIconAsset(i.name))">Remove</button>
      </div>
    </section>
  </ModalDialog>
</template>

<style scoped>
.intro {
  margin: 0 0 10px;
  line-height: 1.5;
}
section {
  border: 1px solid var(--c-border-soft);
  border-radius: 4px;
  padding: 8px 10px;
  margin-bottom: 10px;
}
header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 6px;
}
h3 {
  display: flex;
  align-items: center;
  margin: 0;
  font-size: var(--fs-ui);
  color: var(--c-text-bright);
}
.item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 4px 0;
  border-top: 1px solid var(--c-border-soft);
}
.sample {
  width: 28px;
  font-size: 18px;
  text-align: center;
  color: var(--c-text-bright);
}
.name {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.empty {
  padding: 4px 0;
}
</style>
