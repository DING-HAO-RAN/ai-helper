<template>
  <aside class="sidebar">
    <!-- 顶部导航菜单项 -->
    <div class="nav-section">
      <div class="section-label">CORE FUNCTIONS // 核心功能</div>

      <nav class="nav-list">
        <button
          class="nav-item"
          :class="{ active: currentTab === 'prompts' }"
          @click="$emit('update:currentTab', 'prompts')"
        >
          <div class="nav-glow-bar"></div>
          <Sparkles :size="18" class="nav-icon" />
          <div class="nav-meta">
            <span class="nav-title">提示词宝库</span>
            <span class="nav-sub">PROMPT VAULT</span>
          </div>
        </button>

        <button
          class="nav-item"
          :class="{ active: currentTab === 'tokens' }"
          @click="$emit('update:currentTab', 'tokens')"
        >
          <div class="nav-glow-bar"></div>
          <KeyRound :size="18" class="nav-icon" />
          <div class="nav-meta">
            <span class="nav-title">GitHub 令牌</span>
            <span class="nav-sub">CREDENTIALS</span>
          </div>
        </button>

        <button
          class="nav-item"
          :class="{ active: currentTab === 'agents' }"
          @click="$emit('update:currentTab', 'agents')"
        >
          <div class="nav-glow-bar"></div>
          <Bot :size="18" class="nav-icon" />
          <div class="nav-meta">
            <span class="nav-title">AGENT.md 矩阵</span>
            <span class="nav-sub">SCAN & INJECT</span>
          </div>
        </button>

        <button
          class="nav-item"
          :class="{ active: currentTab === 'timezone' }"
          @click="$emit('update:currentTab', 'timezone')"
        >
          <div class="nav-glow-bar"></div>
          <Globe :size="18" class="nav-icon" />
          <div class="nav-meta">
            <span class="nav-title">Codex 时区</span>
            <span class="nav-sub">PROXY // CHATGPT</span>
          </div>
        </button>
      </nav>
    </div>

    <!-- 底部系统设置与退出区域 -->
    <div class="bottom-section">
      <div class="divider"></div>

      <button class="nav-item settings-btn" @click="$emit('open-settings')">
        <div class="nav-glow-bar"></div>
        <Settings :size="18" class="nav-icon" />
        <div class="nav-meta">
          <span class="nav-title">系统设置</span>
          <span class="nav-sub">SETTINGS // CLI</span>
        </div>
      </button>

      <div class="status-badge-card">
        <div class="badge-row">
          <span class="label">ENGINE:</span>
          <span class="val">RUST 2021</span>
        </div>
        <div class="badge-row">
          <span class="label">SECURITY:</span>
          <span class="val-green">DPAPI ACTIVE</span>
        </div>
      </div>
    </div>
  </aside>
</template>

<script setup lang="ts">
import { Sparkles, KeyRound, Bot, Globe, Settings } from "lucide-vue-next";

defineProps<{
  currentTab: string;
}>();

defineEmits<{
  (e: "update:currentTab", tab: string): void;
  (e: "open-settings"): void;
}>();
</script>

<style scoped>
.sidebar {
  width: 210px;
  min-width: 210px;
  background: rgba(12, 15, 25, 0.96);
  border-right: 1px solid rgba(0, 240, 255, 0.12);
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  padding: 14px 10px;
}

.section-label {
  font-size: 10px;
  font-weight: 700;
  color: var(--cyber-text-muted);
  letter-spacing: 1px;
  margin-bottom: 12px;
  padding-left: 6px;
}

.nav-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.nav-item {
  width: 100%;
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 12px;
  background: transparent;
  border: 1px solid transparent;
  border-radius: 6px;
  cursor: pointer;
  position: relative;
  text-align: left;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  color: #94a3b8;
}

.nav-glow-bar {
  position: absolute;
  left: 0;
  top: 15%;
  bottom: 15%;
  width: 3px;
  background: transparent;
  border-radius: 2px;
  transition: all 0.2s ease;
}

.nav-icon {
  color: #64748b;
  transition: color 0.2s ease;
  flex-shrink: 0;
}

.nav-meta {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.nav-title {
  font-size: 13px;
  font-weight: 600;
  color: #cbd5e1;
}

.nav-sub {
  font-size: 9px;
  font-weight: 700;
  letter-spacing: 0.8px;
  color: #64748b;
}

.nav-item:hover {
  background: rgba(0, 240, 255, 0.06);
  border-color: rgba(0, 240, 255, 0.2);
}

.nav-item:hover .nav-icon {
  color: var(--cyber-neon-cyan);
}

.nav-item:hover .nav-title {
  color: #fff;
}

.nav-item.active {
  background: linear-gradient(90deg, rgba(0, 240, 255, 0.12), rgba(0, 240, 255, 0.02));
  border-color: rgba(0, 240, 255, 0.4);
  box-shadow: 0 0 10px rgba(0, 240, 255, 0.15);
}

.nav-item.active .nav-glow-bar {
  background: var(--cyber-neon-cyan);
  box-shadow: 0 0 8px var(--cyber-neon-cyan);
}

.nav-item.active .nav-icon {
  color: var(--cyber-neon-cyan);
}

.nav-item.active .nav-title {
  color: #fff;
}

.nav-item.active .nav-sub {
  color: var(--cyber-neon-cyan);
}

.bottom-section {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.divider {
  height: 1px;
  background: rgba(0, 240, 255, 0.1);
  margin-bottom: 4px;
}

.status-badge-card {
  background: rgba(8, 10, 18, 0.85);
  border: 1px solid rgba(0, 240, 255, 0.1);
  border-radius: 4px;
  padding: 8px 10px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.badge-row {
  display: flex;
  justify-content: space-between;
  font-size: 10px;
  font-weight: 600;
}

.badge-row .label {
  color: #64748b;
}

.badge-row .val {
  color: #cbd5e1;
}

.badge-row .val-green {
  color: var(--cyber-neon-green);
  text-shadow: 0 0 4px rgba(0, 255, 157, 0.4);
}
</style>
