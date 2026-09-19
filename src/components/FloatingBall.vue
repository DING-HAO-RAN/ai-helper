<template>
  <div class="floating-orb-container" @contextmenu.prevent="handleRightClick">
    <!-- 赛博量子悬浮球核心主体 -->
    <div
      class="orb-body"
      :class="{ 'is-dragging': isDragging, 'copied-active': justCopied }"
      @mousedown="handleMouseDown"
      @dblclick="handleDoubleClick"
      title="左键按住拖动 / 单击恢复控制台 / 右键快捷菜单"
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
  </div>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

const isDragging = ref(false);
const justCopied = ref(false);

let startX = 0;
let startY = 0;
let dragged = false;

// 鼠标按下：左键准备拖动或单击判定
const handleMouseDown = (e: MouseEvent) => {
  if (e.button === 2) {
    // 右键交由 contextmenu 处理
    return;
  }

  if (e.button !== 0) return;

  startX = e.screenX;
  startY = e.screenY;
  dragged = false;

  const onMouseMove = (moveEvent: MouseEvent) => {
    const dist = Math.hypot(moveEvent.screenX - startX, moveEvent.screenY - startY);
    if (dist > 4) {
      dragged = true;
      isDragging.value = true;
      // 移动超过 4px，立即调用 Rust 原生窗口拖拽
      invoke("drag_floating_ball").catch((err) => {
        console.error("拖拽失败:", err);
      });
      cleanup();
    }
  };

  const onMouseUp = () => {
    cleanup();
    isDragging.value = false;
    // 如果没有发生位移，则判定为左键点击，唤醒主控制台
    if (!dragged) {
      invoke("restore_from_floating_ball").catch((err) => {
        console.error("恢复主窗口失败:", err);
      });
    }
  };

  const cleanup = () => {
    window.removeEventListener("mousemove", onMouseMove);
    window.removeEventListener("mouseup", onMouseUp);
  };

  window.addEventListener("mousemove", onMouseMove);
  window.addEventListener("mouseup", onMouseUp);
};

// 双击事件
const handleDoubleClick = () => {
  invoke("restore_from_floating_ball").catch((err) => {
    console.error("双击恢复主窗口失败:", err);
  });
};

// 右键呼出原生上下文菜单
const handleRightClick = async () => {
  try {
    await invoke("show_floating_context_menu");
  } catch (err) {
    console.error("弹出右键快捷菜单失败:", err);
  }
};
</script>

<style scoped>
.floating-orb-container {
  width: 100vw;
  height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  overflow: hidden;
  position: relative;
  user-select: none;
  -webkit-user-select: none;
}

/* 赛博悬浮球主体 */
.orb-body {
  width: 58px;
  height: 58px;
  border-radius: 50%;
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: grab;
  user-select: none;
  background: radial-gradient(circle at 35% 35%, #1a233d, #060912 85%);
  box-shadow: 0 0 16px rgba(0, 240, 255, 0.5), inset 0 0 10px rgba(0, 240, 255, 0.35);
  border: 1.5px solid rgba(0, 240, 255, 0.8);
  transition: transform 0.15s ease, box-shadow 0.2s ease;
}

.orb-body:hover {
  box-shadow: 0 0 22px rgba(0, 240, 255, 0.75), inset 0 0 12px rgba(0, 240, 255, 0.5);
  border-color: #00f0ff;
}

.orb-body:active,
.orb-body.is-dragging {
  cursor: grabbing;
  transform: scale(1.05);
  box-shadow: 0 0 26px rgba(0, 240, 255, 0.9), inset 0 0 14px rgba(0, 240, 255, 0.7);
}

.orb-body.copied-active {
  border-color: var(--cyber-neon-green);
  box-shadow: 0 0 24px rgba(0, 255, 157, 0.85);
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
  background: rgba(0, 240, 255, 0.12);
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
  top: 3px;
  right: 6px;
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background: var(--cyber-neon-green);
  box-shadow: 0 0 8px var(--cyber-neon-green);
  pointer-events: none;
}
</style>
