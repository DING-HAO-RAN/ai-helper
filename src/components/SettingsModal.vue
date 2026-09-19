<template>
  <div v-if="isOpen" class="modal-overlay" @click.self="$emit('close')">
    <div class="cyber-modal cyber-card">
      <div class="modal-header">
        <div class="header-left">
          <Settings class="header-icon" :size="18" />
          <span class="modal-title">SYSTEM SETTINGS // 本地配置与便携式存储</span>
        </div>
        <button class="close-btn" @click="$emit('close')">
          <X :size="16" />
        </button>
      </div>

      <div class="modal-body">
        <!-- 便携式存储文件位置 -->
        <div class="setting-item">
          <div class="item-header">
            <span class="item-title">程序所在目录 (Portable Workdir)</span>
            <button class="cyber-btn mini-open-btn" @click="handleOpenDir(pathsInfo.app_dir)">
              <FolderOpen :size="12" />
              打开程序文件夹
            </button>
          </div>
          <div class="paths-grid">
            <div class="path-row">
              <span class="path-tag">配置 config.json</span>
              <code>{{ pathsInfo.config_path || "加载中..." }}</code>
            </div>
            <div class="path-row">
              <span class="path-tag">数据 storage.json</span>
              <code>{{ pathsInfo.storage_path || "加载中..." }}</code>
            </div>
            <div class="path-row">
              <span class="path-tag">索引 agent_index.json</span>
              <code>{{ pathsInfo.agent_index_path || "加载中..." }}</code>
            </div>
          </div>
        </div>

        <!-- 命令行 AI 工具集成指南 -->
        <div class="setting-item">
          <div class="item-header">
            <span class="item-title">AI 工具命令行调用规范 (CLI)</span>
            <span class="cyber-badge cyber-badge-green">READY</span>
          </div>
          <p class="item-desc">
            无需启动图形界面，终端脚本与外部 AI Agent 可直接调用以下命令毫秒级读取解密后的 GitHub 令牌：
          </p>
          <div class="code-box">
            <div class="code-row">
              <span class="prompt">$</span>
              <span class="cmd">ai-helper token get</span>
              <button class="copy-mini-btn" @click="copyText('ai-helper token get')">复制</button>
            </div>
            <div class="code-row">
              <span class="prompt">PS:</span>
              <span class="cmd">$env:GITHUB_TOKEN = (ai-helper token get)</span>
              <button class="copy-mini-btn" @click="copyText('$env:GITHUB_TOKEN = (ai-helper token get)')">复制</button>
            </div>
            <div class="code-row">
              <span class="prompt">TZ:</span>
              <span class="cmd">ai-helper tz apply</span>
              <button class="copy-mini-btn" @click="copyText('ai-helper tz apply')">复制</button>
            </div>
          </div>
        </div>

        <!-- 悬浮球快捷交互提示 -->
        <div class="setting-item">
          <div class="item-header">
            <span class="item-title">悬浮球交互提示</span>
            <span class="cyber-badge cyber-badge-cyan">WINDOWS 原生集成</span>
          </div>
          <ul class="tips-list">
            <li><b>左键按住拖动</b>：任意平滑拖移至桌面任意边缘位置停靠。</li>
            <li><b>左键单击 / 双击</b>：瞬间唤醒主控制台窗口并自动前置置顶。</li>
            <li><b>鼠标右键单击</b>：呼出系统原生快捷菜单，可快速复制默认 Token 或退出。</li>
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
import { Settings, X, Power, FolderOpen } from "lucide-vue-next";

interface StoragePathsInfo {
  app_dir: string;
  config_path: string;
  storage_path: string;
  agent_index_path: string;
}

const props = defineProps<{
  isOpen: boolean;
}>();

defineEmits<{
  (e: "close"): void;
}>();

const pathsInfo = ref<StoragePathsInfo>({
  app_dir: "",
  config_path: "",
  storage_path: "",
  agent_index_path: "",
});

const fetchPathsInfo = async () => {
  try {
    pathsInfo.value = await invoke<StoragePathsInfo>("get_storage_paths_info");
  } catch (err) {
    console.error("读取路径失败:", err);
  }
};

watch(
  () => props.isOpen,
  (val) => {
    if (val) fetchPathsInfo();
  }
);

onMounted(() => {
  if (props.isOpen) fetchPathsInfo();
});

const handleOpenDir = async (dir: string) => {
  if (!dir) return;
  try {
    await invoke("open_in_explorer", { path: dir });
  } catch (err) {
    alert("打开文件夹失败: " + err);
  }
};

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
  width: 580px;
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
  gap: 14px;
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

.mini-open-btn {
  font-size: 11px;
  padding: 2px 8px;
}

.paths-grid {
  display: flex;
  flex-direction: column;
  gap: 6px;
  background: rgba(8, 11, 20, 0.9);
  padding: 8px 10px;
  border: 1px solid rgba(0, 240, 255, 0.1);
  border-radius: 4px;
}

.path-row {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 11px;
}

.path-tag {
  color: var(--cyber-neon-cyan);
  font-weight: 600;
  width: 140px;
  flex-shrink: 0;
}

.path-row code {
  color: #94a3b8;
  overflow: hidden;
  white-space: nowrap;
  text-overflow: ellipsis;
  font-family: 'Consolas', monospace;
}

.item-desc {
  font-size: 11px;
  color: #94a3b8;
  margin-bottom: 6px;
  line-height: 1.5;
}

.code-box {
  background: rgba(6, 8, 15, 0.95);
  border: 1px solid rgba(0, 240, 255, 0.15);
  border-radius: 4px;
  padding: 8px 12px;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.code-row {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 11px;
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
  padding: 1px 6px;
  border-radius: 3px;
  font-size: 10px;
  cursor: pointer;
}

.copy-mini-btn:hover {
  background: var(--cyber-neon-cyan);
  color: #000;
}

.tips-list {
  padding-left: 18px;
  font-size: 11px;
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
