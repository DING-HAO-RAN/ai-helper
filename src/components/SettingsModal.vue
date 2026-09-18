<template>
  <div v-if="isOpen" class="modal-overlay" @click.self="$emit('close')">
    <div class="cyber-modal cyber-card">
      <div class="modal-header">
        <div class="header-left">
          <Settings class="header-icon" :size="18" />
          <span class="modal-title">SYSTEM SETTINGS // 系统设置</span>
        </div>
        <button class="close-btn" @click="$emit('close')">
          <X :size="16" />
        </button>
      </div>

      <div class="modal-body">
        <!-- 存储路径 -->
        <div class="setting-item">
          <div class="item-header">
            <span class="item-title">本地数据加密存储路径</span>
            <span class="cyber-badge cyber-badge-cyan">DPAPI PROTECTED</span>
          </div>
          <div class="path-display">
            <code>{{ storagePath || "正在读取存储路径..." }}</code>
          </div>
        </div>

        <!-- 命令行 AI 工具集成指南 -->
        <div class="setting-item">
          <div class="item-header">
            <span class="item-title">AI 工具命令行调用规范 (CLI)</span>
            <span class="cyber-badge cyber-badge-green">READY</span>
          </div>
          <p class="item-desc">
            无需启动图形界面，终端与 AI Agent 可直接调用以下命令毫秒级读取解密后的 GitHub 令牌：
          </p>
          <div class="code-box">
            <div class="code-row">
              <span class="prompt">$</span>
              <span class="cmd">ai-helper token get</span>
              <button class="copy-mini-btn" @click="copyText('ai-helper token get')">复制</button>
            </div>
            <div class="code-row">
              <span class="prompt">$env</span>
              <span class="cmd">$env:GITHUB_TOKEN = (ai-helper token get)</span>
              <button class="copy-mini-btn" @click="copyText('$env:GITHUB_TOKEN = (ai-helper token get)')">复制</button>
            </div>
          </div>
        </div>

        <!-- 悬浮球说明 -->
        <div class="setting-item">
          <div class="item-header">
            <span class="item-title">悬浮球交互提示</span>
          </div>
          <ul class="tips-list">
            <li>关闭主窗口时自动缩小为桌面赛博悬浮球，支持全屏自由拖拽。</li>
            <li>鼠标左键点击悬浮球：瞬间还原主控制台。</li>
            <li>鼠标右键点击悬浮球：呼出赛博快捷菜单，可一键复制默认令牌或彻底退出。</li>
          </ul>
        </div>
      </div>

      <div class="modal-footer">
        <button class="cyber-btn cyber-btn-danger" @click="handleExitApp">
          <Power :size="14" />
          彻底退出程序
        </button>
        <button class="cyber-btn cyber-btn-primary" @click="$emit('close')">
          完成并关闭
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { Settings, X, Power } from "lucide-vue-next";

const props = defineProps<{
  isOpen: boolean;
}>();

defineEmits<{
  (e: "close"): void;
}>();

const storagePath = ref("");

const fetchStoragePath = async () => {
  try {
    storagePath.value = await invoke<string>("get_app_storage_location");
  } catch (err) {
    storagePath.value = "读取失败";
  }
};

watch(
  () => props.isOpen,
  (val) => {
    if (val) fetchStoragePath();
  }
);

onMounted(() => {
  if (props.isOpen) fetchStoragePath();
});

const copyText = (txt: string) => {
  navigator.clipboard.writeText(txt);
  alert("命令已复制至剪贴板");
};

const handleExitApp = async () => {
  if (confirm("确定要完全退出 AI Helper 吗？")) {
    await invoke("exit_app");
  }
};
</script>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(4, 6, 12, 0.75);
  backdrop-filter: blur(6px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 999;
}

.cyber-modal {
  width: 540px;
  background: rgba(14, 18, 30, 0.98);
  border: 1px solid rgba(0, 240, 255, 0.35);
  box-shadow: 0 0 25px rgba(0, 240, 255, 0.2);
  display: flex;
  flex-direction: column;
}

.modal-header {
  height: 44px;
  padding: 0 16px;
  border-bottom: 1px solid rgba(0, 240, 255, 0.15);
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.header-icon {
  color: var(--cyber-neon-cyan);
}

.modal-title {
  font-size: 13px;
  font-weight: 700;
  color: #fff;
  letter-spacing: 0.5px;
}

.close-btn {
  background: transparent;
  border: none;
  color: #64748b;
  cursor: pointer;
}

.close-btn:hover {
  color: #fff;
}

.modal-body {
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.item-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 6px;
}

.item-title {
  font-size: 12px;
  font-weight: 700;
  color: #cbd5e1;
}

.path-display {
  background: rgba(8, 11, 20, 0.9);
  padding: 8px 12px;
  border: 1px solid rgba(0, 240, 255, 0.1);
  border-radius: 4px;
  font-size: 12px;
  color: var(--cyber-neon-cyan);
  word-break: break-all;
}

.item-desc {
  font-size: 12px;
  color: #94a3b8;
  margin-bottom: 8px;
  line-height: 1.5;
}

.code-box {
  background: rgba(6, 8, 15, 0.95);
  border: 1px solid rgba(0, 240, 255, 0.15);
  border-radius: 4px;
  padding: 8px 12px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.code-row {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  font-family: 'Consolas', monospace;
}

.prompt {
  color: var(--cyber-neon-pink);
  font-weight: 700;
}

.cmd {
  color: #e2e8f0;
  flex: 1;
}

.copy-mini-btn {
  background: rgba(0, 240, 255, 0.1);
  border: 1px solid rgba(0, 240, 255, 0.3);
  color: var(--cyber-neon-cyan);
  padding: 2px 8px;
  border-radius: 3px;
  font-size: 11px;
  cursor: pointer;
}

.copy-mini-btn:hover {
  background: var(--cyber-neon-cyan);
  color: #000;
}

.tips-list {
  padding-left: 18px;
  font-size: 12px;
  color: #94a3b8;
  line-height: 1.6;
}

.modal-footer {
  padding: 12px 16px;
  border-top: 1px solid rgba(0, 240, 255, 0.12);
  display: flex;
  justify-content: space-between;
}
</style>
