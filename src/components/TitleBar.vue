<template>
  <header
    class="titlebar"
    data-tauri-drag-region="deep"
    @mousedown="handleTitlebarMouseDown"
    @dblclick="handleToggleMaximize"
  >
    <!-- 左侧 Logo 与系统指示灯 -->
    <div class="brand-zone" data-tauri-drag-region="deep">
      <div class="logo-box">
        <img src="/cyber-core.png" alt="Logo" class="logo-img" />
      </div>
      <div class="pulse-indicator">
        <span class="pulse-dot"></span>
        <span class="status-text">SYS_ONLINE</span>
      </div>
      <div class="app-title" data-tauri-drag-region="deep">
        AI HELPER <span class="title-tag">MATRIX v1.0</span>
      </div>
    </div>

    <!-- 中间拖拽扩展区 -->
    <div class="drag-spacer" data-tauri-drag-region="deep"></div>

    <!-- 右侧窗口控制按钮 -->
    <div class="window-controls" data-tauri-drag-region="false" @mousedown.stop>
      <button class="win-btn win-min" title="最小化" @click="handleMinimize">
        <Minus :size="14" />
      </button>
      <button class="win-btn win-max" title="最大化 / 还原" @click="handleToggleMaximize">
        <Square :size="12" />
      </button>
      <button class="win-btn win-close" title="缩小至赛博悬浮球" @click="handleCloseToOrb">
        <X :size="14" />
      </button>
    </div>
  </header>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { Minus, Square, X } from "lucide-vue-next";

const isMaximized = ref(false);

// 鼠标左键按下标题栏任何空白处时触发 Windows 原生窗口拖拽
const handleTitlebarMouseDown = (e: MouseEvent) => {
  if (e.button === 0 && !(e.target as HTMLElement).closest(".window-controls")) {
    invoke("drag_window").catch((err) => {
      console.error("拖拽窗口失败:", err);
    });
  }
};

const handleMinimize = async () => {
  try {
    await invoke("minimize_main_window");
  } catch (err) {
    console.error("最小化失败:", err);
  }
};

const handleToggleMaximize = async () => {
  try {
    const result = await invoke<boolean>("maximize_or_restore_main_window");
    isMaximized.value = result;
  } catch (err) {
    console.error("切换最大化失败:", err);
  }
};

const handleCloseToOrb = async () => {
  try {
    // 隐藏主窗口并呼出桌面悬浮球
    await invoke("close_to_floating_ball");
  } catch (err) {
    console.error("切换至悬浮球失败:", err);
  }
};
</script>

<style scoped>
.titlebar {
  height: 38px;
  background: rgba(10, 13, 22, 0.95);
  border-bottom: 1px solid rgba(0, 240, 255, 0.15);
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 10px;
  user-select: none;
  -webkit-user-select: none;
  cursor: default;
}

.brand-zone {
  display: flex;
  align-items: center;
  gap: 10px;
}

.logo-box {
  width: 22px;
  height: 22px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.logo-img {
  width: 20px;
  height: 20px;
  border-radius: 4px;
  box-shadow: 0 0 6px rgba(0, 240, 255, 0.5);
}

.pulse-indicator {
  display: flex;
  align-items: center;
  gap: 5px;
  padding: 2px 6px;
  background: rgba(0, 255, 157, 0.08);
  border: 1px solid rgba(0, 255, 157, 0.2);
  border-radius: 3px;
}

.status-text {
  font-size: 10px;
  font-weight: 700;
  color: var(--cyber-neon-green);
  letter-spacing: 0.5px;
}

.app-title {
  font-size: 12px;
  font-weight: 800;
  color: #fff;
  letter-spacing: 1px;
}

.title-tag {
  font-size: 10px;
  font-weight: 600;
  color: var(--cyber-neon-cyan);
  margin-left: 4px;
}

.drag-spacer {
  flex: 1;
  height: 100%;
}

.window-controls {
  display: flex;
  align-items: center;
  gap: 4px;
}

.win-btn {
  width: 32px;
  height: 26px;
  background: transparent;
  border: 1px solid transparent;
  border-radius: 4px;
  color: #94a3b8;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: all 0.15s ease;
}

.win-btn:hover {
  background: rgba(255, 255, 255, 0.08);
  color: #fff;
}

.win-btn.win-close:hover {
  background: rgba(255, 0, 85, 0.35);
  color: #fff;
  border-color: var(--cyber-neon-pink);
  box-shadow: 0 0 8px rgba(255, 0, 85, 0.5);
}
</style>
