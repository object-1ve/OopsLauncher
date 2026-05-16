<template>
  <div class="settings-section">
    <div class="about-card">
      <div class="about-brand">
        <div class="about-logo-wrap">
          <img src="@/assets/icon.png" alt="OopsLauncher" class="about-logo" />
        </div>
        <h1 class="app-name">OopsLauncher</h1>
        <div class="version-chip">Version {{ appVersion }}</div>
        <p class="app-desc">一个极简、高效的 Tauri 启动器</p>
      </div>
      <div class="about-divider">
        <span class="divider-dot"></span>
      </div>
      <div class="about-links">
        <a href="https://github.com/yourname/OopsLauncher" target="_blank" class="about-link" rel="noopener">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M9 19c-5 1.5-5-2.5-7-3m14 6v-3.87a3.37 3.37 0 0 0-.94-2.61c3.14-.35 6.44-1.54 6.44-7A5.44 5.44 0 0 0 20 4.77 5.07 5.07 0 0 0 19.91 1S18.73.65 16 2.48a13.38 13.38 0 0 0-7 0C6.27.65 5.09 1 5.09 1A5.07 5.07 0 0 0 5 4.77a5.44 5.44 0 0 0-1.5 3.78c0 5.42 3.3 6.61 6.44 7A3.37 3.37 0 0 0 9 18.13V22"/>
          </svg>
          <span>GitHub 仓库</span>
          <svg class="link-arrow" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M7 17l9-9M7 8h9v9"/></svg>
        </a>
        <a href="#" class="about-link" @click.prevent>
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="23 4 23 10 17 10"/><path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"/>
          </svg>
          <span>检查更新</span>
          <svg class="link-arrow" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M7 17l9-9M7 8h9v9"/></svg>
        </a>
      </div>
      <div class="about-footer">
        <span class="footer-text">Made with Tauri &amp; Vue</span>
      </div>
    </div>
  </div>
</template>

<script setup>
defineProps({
  appVersion: { type: String, required: true }
});
</script>

<style scoped>
.settings-section {
  animation: sectionEnter 0.4s ease both;
  display: flex;
  justify-content: center;
  padding-top: 20px;
}

.about-card {
  background: var(--settings-card-bg);
  border: 1px solid var(--settings-card-border);
  border-radius: var(--settings-radius-lg);
  padding: 40px 36px 28px;
  box-shadow: var(--settings-shadow-sm);
  text-align: center;
  width: 100%;
  max-width: 420px;
  transition: box-shadow var(--settings-transition);
}

.about-card:hover {
  box-shadow: var(--settings-shadow-md);
}

.about-brand {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
}

.about-logo-wrap {
  width: 88px;
  height: 88px;
  border-radius: 22px;
  background: linear-gradient(135deg, var(--el-color-primary-light-5, #a0cfff), var(--el-color-primary, #409eff));
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 4px;
  box-shadow: 0 8px 24px rgba(64, 158, 255, 0.25);
  margin-bottom: 4px;
}

.about-logo {
  width: 100%;
  height: 100%;
  border-radius: 18px;
  object-fit: contain;
  background: #fff;
}

.app-name {
  font-size: 26px;
  font-weight: 700;
  color: var(--settings-text-primary);
  letter-spacing: -0.02em;
  line-height: 1.2;
}

.version-chip {
  display: inline-block;
  font-size: 12px;
  font-weight: 600;
  color: var(--settings-text-muted);
  background: #f4f5f9;
  padding: 4px 14px;
  border-radius: 20px;
  letter-spacing: 0.03em;
}

.app-desc {
  font-size: 14px;
  color: var(--settings-text-secondary);
  line-height: 1.5;
  margin-top: 4px;
}

.about-divider {
  display: flex;
  align-items: center;
  justify-content: center;
  margin: 24px 0;
  height: 1px;
  background: var(--settings-border);
  position: relative;
}

.divider-dot {
  position: absolute;
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--settings-border);
}

.about-links {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.about-link {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 16px;
  border-radius: var(--settings-radius-sm);
  color: var(--settings-text-secondary);
  text-decoration: none;
  font-size: 14px;
  font-weight: 500;
  transition: all 0.2s;
  cursor: pointer;
}

.about-link:hover {
  background: rgba(0, 0, 0, 0.04);
  color: var(--el-color-primary);
}

.about-link:hover .link-arrow {
  transform: translate(2px, -2px);
  opacity: 1;
}

.link-arrow {
  margin-left: auto;
  opacity: 0.4;
  transition: all 0.2s;
}

.about-footer {
  margin-top: 24px;
  padding-top: 16px;
  border-top: 1px solid var(--settings-border);
}

.footer-text {
  font-size: 11px;
  color: var(--settings-text-muted);
  letter-spacing: 0.04em;
}

@keyframes sectionEnter {
  from { opacity: 0; transform: translateY(12px); }
  to { opacity: 1; transform: translateY(0); }
}
</style>
