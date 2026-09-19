<template>
  <!-- 若为悬浮球窗口路由，直接渲染悬浮球 -->
  <FloatingBall v-if="isFloatingOrb" />

  <!-- 主窗口布局 -->
  <div v-else class="app-layout">
    <TitleBar />

    <div class="main-body">
      <Sidebar
        v-model:currentTab="activeTab"
        @open-settings="isSettingsOpen = true"
      />

      <main class="content-area">
        <PromptVault v-if="activeTab === 'prompts'" />
        <TokenVault v-else-if="activeTab === 'tokens'" />
        <AgentManager v-else-if="activeTab === 'agents'" />
        <TimezoneManager v-else-if="activeTab === 'timezone'" />
        <AntigravityRepair v-else-if="activeTab === 'antigravity'" />
      </main>
    </div>

    <SettingsModal
      :isOpen="isSettingsOpen"
      @close="isSettingsOpen = false"
    />

    <!-- 全局统一赛博朋克模态提示与确认框 -->
    <CyberDialog />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from "vue";
import TitleBar from "./components/TitleBar.vue";
import Sidebar from "./components/Sidebar.vue";
import SettingsModal from "./components/SettingsModal.vue";
import CyberDialog from "./components/CyberDialog.vue";
import PromptVault from "./components/PromptVault.vue";
import TokenVault from "./components/TokenVault.vue";
import AgentManager from "./components/AgentManager.vue";
import TimezoneManager from "./components/TimezoneManager.vue";
import AntigravityRepair from "./components/AntigravityRepair.vue";
import FloatingBall from "./components/FloatingBall.vue";

const isFloatingOrb = ref(false);
const activeTab = ref("prompts");
const isSettingsOpen = ref(false);

onMounted(() => {
  // 根据 URL Hash 判断是否为桌面悬浮球窗口
  if (window.location.hash.includes("floating")) {
    isFloatingOrb.value = true;
  }
});
</script>

<style scoped>
.app-layout {
  display: flex;
  flex-direction: column;
  width: 100vw;
  height: 100vh;
  overflow: hidden;
  background: radial-gradient(circle at 80% 20%, rgba(16, 26, 45, 0.7) 0%, #08090f 75%);
  border: 1px solid rgba(0, 240, 255, 0.2);
}

.main-body {
  flex: 1;
  display: flex;
  overflow: hidden;
  position: relative;
}

.content-area {
  flex: 1;
  height: 100%;
  overflow: hidden;
  background: rgba(10, 12, 20, 0.6);
  backdrop-filter: blur(8px);
  display: flex;
  flex-direction: column;
}
</style>
