<script setup lang="ts">
import ModalDialog from './ModalDialog.vue'
import { APP } from '../../core/about'
import GitHubLink from '../ui/GitHubLink.vue'
import { useEditor } from '../../stores/editor'

const store = useEditor()
</script>

<template>
  <ModalDialog :title="`About ${APP.name}`" :width="440" ok-label="Close" :cancel="false" @ok="store.state.dialog = null">
    <div class="about" data-testid="about">
      <svg viewBox="0 0 32 32" width="64" height="64">
        <rect width="32" height="32" rx="6" fill="#0d2238" />
        <text x="16" y="21.5" text-anchor="middle" font-size="14" font-weight="700" fill="#31a8ff" font-family="Segoe UI, Arial">Td</text>
      </svg>
      <div class="info">
        <h2>{{ APP.name }}</h2>
        <p class="dim">Version {{ APP.version }}</p>
        <p>A grid based editor for clean, topologically correct metro and transit maps: draw lines, place stations, labels and pictograms, then export to SVG.</p>
        <dl>
          <dt>Author</dt>
          <dd><a :href="APP.authorUrl" target="_blank" rel="noopener">{{ APP.author }}</a></dd>
          <dt>Source</dt>
          <dd><a :href="APP.repoUrl" target="_blank" rel="noopener">{{ APP.repoUrl.replace('https://', '') }}</a></dd>
          <dt>Feedback</dt>
          <dd><a :href="APP.issuesUrl" target="_blank" rel="noopener">Report a bug or request a feature</a></dd>
          <dt>License</dt>
          <dd>{{ APP.license }}</dd>
        </dl>
        <GitHubLink class="repo" :size="22" caption="View the source on GitHub" data-testid="about-github" />
      </div>
    </div>
  </ModalDialog>
</template>

<style scoped>
.about {
  display: flex;
  gap: 16px;
  line-height: 1.5;
}
.info {
  min-width: 0;
}
h2 {
  margin: 0;
  font-size: 16px;
  color: var(--c-text-bright);
}
p {
  margin: 4px 0;
}
dl {
  display: grid;
  grid-template-columns: auto 1fr;
  gap: 2px 12px;
  margin: 10px 0 0;
}
dt {
  color: var(--c-text-dim);
}
dd {
  margin: 0;
  overflow-wrap: anywhere;
}
a {
  color: var(--c-accent);
}
.repo {
  margin-top: 14px;
}
</style>
