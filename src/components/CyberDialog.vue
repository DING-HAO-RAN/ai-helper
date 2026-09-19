<template>
  <!-- 全局统一赛博朋克模态框 -->
  <div v-if="dialogState.isOpen" class="cyber-dialog-overlay" @click.self="handleCancel">
    <div
      class="cyber-dialog cyber-card"
      :class="{
        'dialog-info': dialogState.type === 'info',
        'dialog-success': dialogState.type === 'success',
        'dialog-warning': dialogState.type === 'warning',
        'dialog-error': dialogState.type === 'error',
      }"
    >
      <!-- 顶部科技发光装饰灯 -->
      <div class="header-glow-strip"></div>

      <!-- 弹窗头部 -->
      <div class="dialog-header">
        <div class="header-left">
          <Info v-if="dialogState.type === 'info'" :size="18" class="icon-info" />
          <CheckCircle2 v-else-if="dialogState.type === 'success'" :size="18" class="icon-success" />
          <AlertTriangle v-else-if="dialogState.type === 'warning'" :size="18" class="icon-warning" />
          <AlertOctagon v-else-if="dialogState.type === 'error'" :size="18" class="icon-error" />
          <span class="dialog-title">{{ dialogState.title }}</span>
        </div>
        <button class="dialog-close-btn" @click="handleCancel">
          <X :size="16" />
        </button>
      </div>

      <!-- 弹窗内容区域 -->
      <div class="dialog-body">
        <p class="dialog-message">{{ dialogState.message }}</p>
      </div>

      <!-- 底部操作按钮 -->
      <div class="dialog-footer">
        <button
          v-if="dialogState.isConfirm"
          class="cyber-btn"
          @click="handleCancel"
        >
          {{ dialogState.cancelText }}
        </button>

        <button
          class="cyber-btn"
          :class="dialogState.type === 'error' ? 'cyber-btn-danger' : 'cyber-btn-primary'"
          @click="handleConfirm"
        >
          {{ dialogState.confirmText }}
        </button>
      </div>
    </div>
  </div>

  <!-- 全局右上角轻量 Toast 通知列表 -->
  <div class="cyber-toast-container">
    <transition-group name="toast-slide">
      <div
        v-for="t in toastList"
        :key="t.id"
        class="cyber-toast-item cyber-card"
        :class="`toast-${t.type}`"
      >
        <CheckCircle2 v-if="t.type === 'success'" :size="15" class="toast-icon-green" />
        <AlertTriangle v-else-if="t.type === 'warning'" :size="15" class="toast-icon-yellow" />
        <AlertOctagon v-else-if="t.type === 'error'" :size="15" class="toast-icon-pink" />
        <Info v-else :size="15" class="toast-icon-cyan" />
        <span class="toast-text">{{ t.message }}</span>
      </div>
    </transition-group>
  </div>
</template>

<script setup lang="ts">
import { dialogState, toastList } from "../utils/dialog";
import {
  Info,
  CheckCircle2,
  AlertTriangle,
  AlertOctagon,
  X,
} from "lucide-vue-next";

const handleConfirm = () => {
  dialogState.isOpen = false;
  if (dialogState.resolve) {
    dialogState.resolve(true);
  }
};

const handleCancel = () => {
  dialogState.isOpen = false;
  if (dialogState.resolve) {
    dialogState.resolve(false);
  }
};
</script>

<style scoped>
.cyber-dialog-overlay {
  position: fixed;
  inset: 0;
  background: rgba(4, 6, 12, 0.78);
  backdrop-filter: blur(8px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 99999;
  animation: fade-in 0.15s ease-out;
}

@keyframes fade-in {
  from { opacity: 0; }
  to { opacity: 1; }
}

.cyber-dialog {
  width: 440px;
  max-width: 90vw;
  background: rgba(14, 18, 30, 0.98);
  border: 1px solid rgba(0, 240, 255, 0.4);
  box-shadow: 0 0 30px rgba(0, 240, 255, 0.25);
  border-radius: 6px;
  overflow: hidden;
  position: relative;
  display: flex;
  flex-direction: column;
}

.dialog-warning {
  border-color: rgba(252, 238, 10, 0.5);
  box-shadow: 0 0 30px rgba(252, 238, 10, 0.2);
}

.dialog-error {
  border-color: rgba(255, 0, 85, 0.5);
  box-shadow: 0 0 30px rgba(255, 0, 85, 0.25);
}

.dialog-success {
  border-color: rgba(0, 255, 157, 0.5);
  box-shadow: 0 0 30px rgba(0, 255, 157, 0.2);
}

.header-glow-strip {
  height: 2px;
  background: linear-gradient(90deg, transparent, var(--cyber-neon-cyan), transparent);
}

.dialog-warning .header-glow-strip {
  background: linear-gradient(90deg, transparent, var(--cyber-neon-yellow), transparent);
}

.dialog-error .header-glow-strip {
  background: linear-gradient(90deg, transparent, var(--cyber-neon-pink), transparent);
}

.dialog-header {
  height: 42px;
  padding: 0 16px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  border-bottom: 1px solid rgba(0, 240, 255, 0.15);
}

.header-left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.dialog-title {
  font-size: 13px;
  font-weight: 700;
  color: #fff;
  letter-spacing: 0.5px;
}

.icon-info { color: var(--cyber-neon-cyan); }
.icon-success { color: var(--cyber-neon-green); }
.icon-warning { color: var(--cyber-neon-yellow); }
.icon-error { color: var(--cyber-neon-pink); }

.dialog-close-btn {
  background: transparent;
  border: none;
  color: #64748b;
  cursor: pointer;
  display: flex;
  align-items: center;
}

.dialog-close-btn:hover {
  color: #fff;
}

.dialog-body {
  padding: 18px 16px;
  max-height: 50vh;
  overflow-y: auto;
}

.dialog-message {
  font-size: 13px;
  color: #cbd5e1;
  line-height: 1.6;
  white-space: pre-wrap;
  word-break: break-all;
}

.dialog-footer {
  padding: 12px 16px;
  border-top: 1px solid rgba(0, 240, 255, 0.12);
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 10px;
}

/* Toast 浮动气泡通知 */
.cyber-toast-container {
  position: fixed;
  top: 48px;
  right: 16px;
  display: flex;
  flex-direction: column;
  gap: 8px;
  z-index: 100000;
  pointer-events: none;
}

.cyber-toast-item {
  padding: 8px 14px;
  background: rgba(12, 16, 28, 0.95);
  border: 1px solid rgba(0, 240, 255, 0.35);
  box-shadow: 0 0 16px rgba(0, 240, 255, 0.25);
  border-radius: 4px;
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  color: #e2e8f0;
  pointer-events: auto;
}

.toast-icon-green { color: var(--cyber-neon-green); }
.toast-icon-yellow { color: var(--cyber-neon-yellow); }
.toast-icon-pink { color: var(--cyber-neon-pink); }
.toast-icon-cyan { color: var(--cyber-neon-cyan); }

.toast-slide-enter-active,
.toast-slide-leave-active {
  transition: all 0.25s ease;
}

.toast-slide-enter-from {
  opacity: 0;
  transform: translateX(40px);
}

.toast-slide-leave-to {
  opacity: 0;
  transform: translateY(-20px);
}
</style>
