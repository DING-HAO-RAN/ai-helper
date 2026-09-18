<template>
  <div class="floating-orb-container" @contextmenu.prevent="handleContextMenu">
    <!-- 赛博量子悬浮球核心主体 -->
    <div
      class="orb-body"
      :class="{ 'is-dragging': isDragging, 'copied-active': justCopied }"
      @mousedown="handleMouseDown"
    >
      <!-- 外部动态旋转霓虹光圈 -->
      <div class="neon-ring"></div>
      <div class="neon-ring-inner"></div>

      <!-- 内部脉冲核心 -->
      <div class="orb-core">
        <img src="/cyber-core.png" alt="AI Core" class="core-icon" />
      </div>

      <!-- 状态小绿灯 -->
      <div class="orb-dot"></div>
    </div>

    <!-- 右键赛博朋克快捷菜单 -->
    <div
      v-if="showMenu"
      class="cyber-context-menu cyber-card"
      @click.stop
    >
      <div class="menu-header">
        <span class="menu-title">AI HELPER // QUICK MENU</span>
      </div>

      <div class="menu-items">
        <button class="menu-item" @click="handleCopyDefaultToken">
          <Copy :size="14" class="menu-icon" />
          <span>{{ justCopied ? "✓ 已复制默认令牌" : "复制默认 GitHub Token" }}</span>
        </button>

        <button class="menu-item" @click="handleRestoreMainWindow">
          <Maximize2 :size="14" class="menu-icon" />
          <span>打开主控制台</span>
        </button>

        <div class="menu-divider"></div>

        <button class="menu-item menu-danger" @click="handleExitApp">
          <Power :size="14" class="menu-icon" />
          <span>彻底退出程序</span>
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { LogicalSize } from "@tauri-apps/api/dpi";
import { Copy, Maximize2, Power } from "lucide-vue-next";

const showMenu = ref(false);
const isDragging = ref(false);
const justCopied = ref(false);

let startX = 0;
let startY = 0;
let dragOccurred = false;

// 动态适应菜单尺寸
watch(showMenu, async (open) => {
  const currentWin = getCurrentWebviewWindow();
  if (open) {
    await currentWin.setSize(new LogicalSize(240, 240));
  } else {
    await currentWin.setSize(new LogicalSize(72, 72));
  }
});

// 鼠标按下：准备拖动
const handleMouseDown = async (e: MouseEvent) => {
  if (e.button === 2) {
    // 右键由 contextmenu 处理
    return;
  }

  startX = e.screenX;
  startY = e.screenY;
  dragOccurred = false;

  const currentWin = getCurrentWebviewWindow();

  // 监听移动与抬起
  const onMouseMove = (moveEvent: MouseEvent) => {
    const dist = Math.hypot(moveEvent.screenX - startX, moveEvent.screenY - startY);
    if (dist > 5) {
      dragOccurred = true;
      isDragging.value = true;
      showMenu.value = false;
      currentWin.startDragging();
      cleanup();
    }
  };

  const onMouseUp = async () => {
    cleanup();
    isDragging.value = false;
    // 如果没有发生拖拽，则判定为左键点击，唤醒主窗口
    if (!dragOccurred) {
      await handleRestoreMainWindow();
    }
  };

  const cleanup = () => {
    window.removeEventListener("mousemove", onMouseMove);
    window.removeEventListener("mouseup", onMouseUp);
  };

  window.addEventListener("mousemove", onMouseMove);
  window.addEventListener("mouseup", onMouseUp);
};

// 右键呼出快捷菜单
const handleContextMenu = () => {
  showMenu.value = !showMenu.value;
};

// 点击空白关闭菜单
const handleWindowClick = () => {
  if (showMenu.value) {
    showMenu.value = false;
  }
};

onMounted(() => {
  window.addEventListener("click", handleWindowClick);
});

onUnmounted(() => {
  window.removeEventListener("click", handleWindowClick);
});

// 唤醒主窗口
const handleRestoreMainWindow = async () => {
  showMenu.value = false;
  try {
    await invoke("restore_from_floating_ball");
  } catch (err) {
    console.error("恢复主窗口失败:", err);
  }
};

// 复制默认 GitHub 令牌
const handleCopyDefaultToken = async () => {
  try {
    const plain = await invoke<string>("get_token_plain_text", { alias: null });
    await navigator.clipboard.writeText(plain);
    justCopied.value = true;
    setTimeout(() => {
      justCopied.value = false;
      showMenu.value = false;
    }, 1500);
  } catch (err) {
    alert("复制失败: " + err);
  }
};

// 彻底退出程序
const handleExitApp = async () => {
  try {
    await invoke("exit_app");
  } catch (err) {
    console.error("退出失败:", err);
  }
};
</script>

<style scoped>
.floating-orb-container {
  width: 100vw;
  height: 100vh;
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  justify-content: flex-start;
  padding: 7px;
  background: transparent;
  overflow: visible;
  position: relative;
}

/* 赛博悬浮球主体 */
.orb-body {
  width: 56px;
  height: 56px;
  border-radius: 50%;
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: grab;
  user-select: none;
  background: radial-gradient(circle at 35% 35%, #18223c, #060912 80%);
  box-shadow: 0 0 16px rgba(0, 240, 255, 0.45), inset 0 0 10px rgba(0, 240, 255, 0.3);
  border: 1.5px solid rgba(0, 240, 255, 0.7);
  transition: transform 0.15s ease, box-shadow 0.2s ease;
}

.orb-body:active,
.orb-body.is-dragging {
  cursor: grabbing;
  transform: scale(1.06);
  box-shadow: 0 0 24px rgba(0, 240, 255, 0.8), inset 0 0 14px rgba(0, 240, 255, 0.6);
}

.orb-body.copied-active {
  border-color: var(--cyber-neon-green);
  box-shadow: 0 0 24px rgba(0, 255, 157, 0.8);
}

/* 霓虹动态旋转光圈 */
.neon-ring {
  position: absolute;
  inset: -4px;
  border-radius: 50%;
  border: 2px dashed rgba(0, 240, 255, 0.65);
  animation: orb-spin 12s linear infinite;
  pointer-events: none;
}

.neon-ring-inner {
  position: absolute;
  inset: -7px;
  border-radius: 50%;
  border: 1.5px dotted rgba(255, 0, 85, 0.5);
  animation: orb-spin-reverse 18s linear infinite;
  pointer-events: none;
}

@keyframes orb-spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

@keyframes orb-spin-reverse {
  from { transform: rotate(360deg); }
  to { transform: rotate(0deg); }
}

/* 内部核心 */
.orb-core {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
  background: rgba(0, 240, 255, 0.1);
  box-shadow: 0 0 10px rgba(0, 240, 255, 0.5);
  pointer-events: none;
}

.core-icon {
  width: 28px;
  height: 28px;
  object-fit: cover;
  filter: drop-shadow(0 0 4px #00f0ff);
}

.orb-dot {
  position: absolute;
  top: 4px;
  right: 6px;
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background: var(--cyber-neon-green);
  box-shadow: 0 0 8px var(--cyber-neon-green);
  pointer-events: none;
}

/* 右键上下文菜单 */
.cyber-context-menu {
  position: absolute;
  top: 68px;
  left: 6px;
  width: 215px;
  background: rgba(12, 16, 28, 0.98);
  border: 1px solid rgba(0, 240, 255, 0.4);
  box-shadow: 0 0 20px rgba(0, 240, 255, 0.3);
  border-radius: 6px;
  padding: 6px 0;
  z-index: 10000;
}

.menu-header {
  padding: 4px 12px 6px;
  border-bottom: 1px solid rgba(0, 240, 255, 0.15);
}

.menu-title {
  font-size: 10px;
  font-weight: 700;
  color: var(--cyber-neon-cyan);
  letter-spacing: 0.5px;
}

.menu-items {
  display: flex;
  flex-direction: column;
  padding: 4px;
}

.menu-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 10px;
  background: transparent;
  border: none;
  color: #cbd5e1;
  font-size: 12px;
  font-weight: 600;
  border-radius: 4px;
  cursor: pointer;
  text-align: left;
  transition: all 0.15s ease;
}

.menu-item:hover {
  background: rgba(0, 240, 255, 0.12);
  color: #fff;
}

.menu-icon {
  color: var(--cyber-neon-cyan);
}

.menu-divider {
  height: 1px;
  background: rgba(255, 255, 255, 0.08);
  margin: 4px 6px;
}

.menu-item.menu-danger {
  color: var(--cyber-neon-pink);
}

.menu-item.menu-danger:hover {
  background: rgba(255, 0, 85, 0.2);
  color: #fff;
}

.menu-item.menu-danger .menu-icon {
  color: var(--cyber-neon-pink);
}
</style>
