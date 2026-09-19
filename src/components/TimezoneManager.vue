<template>
  <div class="timezone-manager-root">
    <!-- 顶部：代理 IP 出口节点与智能时区侦测卡片 -->
    <section class="proxy-radar-card cyber-card">
      <div class="card-header">
        <div class="header-left">
          <Globe class="neon-icon" :size="18" />
          <span class="card-title">PROXY GEO RADAR // 代理节点与时区智能侦测</span>
          <span v-if="isDetecting" class="cyber-badge cyber-badge-yellow">正在侦测中...</span>
          <span v-else-if="proxyInfo" class="cyber-badge cyber-badge-green">侦测完成</span>
        </div>
        <button
          class="cyber-btn"
          :disabled="isDetecting"
          title="重新发起网络探测获取最新代理节点时区"
          @click="handleDetectProxy"
        >
          <Radar :size="14" :class="{ 'spin-anim': isDetecting }" />
          <span>重新侦测代理</span>
        </button>
      </div>

      <div class="geo-display-grid">
        <div class="geo-pill">
          <span class="pill-label">代理出口 IP:</span>
          <span class="pill-val ip-val">{{ proxyInfo?.ip || "侦测中..." }}</span>
        </div>

        <div class="geo-pill">
          <span class="pill-label">地理归属:</span>
          <span class="pill-val">{{ proxyInfo ? `${proxyInfo.country} (${proxyInfo.city})` : "未知" }}</span>
        </div>

        <div class="geo-pill highlight-pill">
          <span class="pill-label">推荐代理时区:</span>
          <span class="pill-val val-cyan">{{ proxyInfo?.timezone_id || "检测中..." }}</span>
        </div>

        <div class="geo-pill">
          <span class="pill-label">Windows 时区名:</span>
          <span class="pill-val">{{ proxyInfo?.windows_tz_name || "检测中..." }}</span>
        </div>
      </div>
    </section>

    <!-- 中部：ChatGPT / Codex 进程状态与隔离保护看板 -->
    <section class="status-board-card cyber-card">
      <div class="board-header">
        <div class="header-left">
          <ShieldAlert class="neon-icon" :size="16" />
          <span class="card-title">CODEX & CHATGPT TARGET // 目标客户端状态</span>
          <span class="cyber-badge cyber-badge-cyan">时区隔离保护（不改系统时区）</span>
        </div>
        <button class="cyber-btn mini-btn" @click="fetchStatus">
          <RefreshCw :size="12" />
          刷新状态
        </button>
      </div>

      <div class="status-grid">
        <div class="status-col">
          <span class="col-title">ChatGPT (Codex) 进程</span>
          <div class="status-tag-row">
            <span
              class="cyber-badge"
              :class="chatgptStatus?.is_running ? 'cyber-badge-green' : 'cyber-badge-yellow'"
            >
              {{ chatgptStatus?.is_running ? `● 运行中 (${chatgptStatus.pids.length} 进程)` : "○ 未运行" }}
            </span>
            <span v-if="chatgptStatus?.cdp_available" class="cyber-badge cyber-badge-cyan">
              ⚡ CDP 注入通道已开启 (9222)
            </span>
          </div>
          <div class="sub-hint">
            物理路径: {{ chatgptStatus?.app_path || "正在自动侦测 WindowsApps 安装目录..." }}
          </div>
        </div>

        <div class="status-col">
          <span class="col-title">当前 ChatGPT 生效时区</span>
          <div class="status-tag-row">
            <span class="tz-active-text">
              {{ chatgptStatus?.active_timezone || selectedTimezone || "跟随系统默认" }}
            </span>
            <span v-if="chatgptStatus?.active_timezone" class="cyber-badge cyber-badge-green">
              ✓ 已独立注入
            </span>
          </div>
          <div class="sub-hint">
            Windows 系统全局时区: <b class="val-green">{{ chatgptStatus?.system_timezone || "China Standard Time" }}</b>（未受影响）
          </div>
        </div>
      </div>
    </section>

    <!-- 下部：时区选择与注入操作区 -->
    <section class="action-panel-card cyber-card">
      <div class="panel-header">
        <Sparkles class="neon-icon" :size="16" />
        <span class="card-title">TIMEZONE INJECTION CONTROL // 注入与控制</span>
      </div>

      <div class="presets-section">
        <div class="presets-title">快捷预设节点时区（点击快速选取）：</div>
        <div class="presets-list">
          <button
            v-if="proxyInfo"
            class="preset-btn proxy-preset-btn"
            :class="{ active: selectedTimezone === proxyInfo.timezone_id }"
            @click="selectedTimezone = proxyInfo.timezone_id"
          >
            <Zap :size="13" />
            <span>★ 代理原生: {{ proxyInfo.timezone_id }}</span>
          </button>

          <button
            v-for="p in presets"
            :key="p.id"
            class="preset-btn"
            :class="{ active: selectedTimezone === p.id }"
            @click="selectedTimezone = p.id"
          >
            <span class="p-region">[{{ p.region }}]</span>
            <span class="p-label">{{ p.label }}</span>
          </button>
        </div>
      </div>

      <!-- 自定义时区输入框 -->
      <div class="custom-input-row">
        <label class="input-label">目标 IANA 时区标识:</label>
        <div class="input-wrapper">
          <input
            v-model="selectedTimezone"
            type="text"
            class="cyber-input"
            placeholder="例如: America/Los_Angeles, America/New_York, Asia/Tokyo..."
          />
        </div>
      </div>

      <!-- 操作按钮群 -->
      <div class="actions-footer">
        <div class="action-left">
          <button
            class="cyber-btn cyber-btn-primary launch-inject-btn"
            :disabled="isOperating || !selectedTimezone"
            title="关闭旧进程，以 CDP 调试通道启动 ChatGPT 并将目标时区注入到所有页面"
            @click="handleLaunchWithTimezone"
          >
            <Power :size="15" />
            <span>{{ isOperating ? "正在注入启动..." : "⚡ 启动并独立注入 ChatGPT (推荐)" }}</span>
          </button>

          <button
            v-if="chatgptStatus?.cdp_available"
            class="cyber-btn"
            :disabled="isOperating || !selectedTimezone"
            title="对当前运行中的 ChatGPT 所有网页即时热更新时区"
            @click="handleInjectRunning"
          >
            <RefreshCw :size="14" />
            <span>热更新当前运行中的时区</span>
          </button>
        </div>

        <div class="action-right">
          <button
            class="cyber-btn"
            title="将整个 Windows 系统时区临时切换为代理时区"
            @click="handleAlignSystemTz"
          >
            <Monitor :size="14" />
            <span>临时对齐系统时区</span>
          </button>

          <button
            class="cyber-btn cyber-btn-danger"
            title="将 Windows 系统时区恢复为原先的中国标准时间"
            @click="handleRestoreSystemTz"
          >
            <RotateCcw :size="14" />
            <span>恢复系统原始时区</span>
          </button>
        </div>
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import {
  Globe,
  Radar,
  ShieldAlert,
  RefreshCw,
  Sparkles,
  Zap,
  Power,
  Monitor,
  RotateCcw,
} from "lucide-vue-next";
import type { ProxyGeoInfo, ChatGPTStatus, TimezonePreset } from "../types/timezone";

const proxyInfo = ref<ProxyGeoInfo | null>(null);
const chatgptStatus = ref<ChatGPTStatus | null>(null);
const presets = ref<TimezonePreset[]>([]);
const selectedTimezone = ref("America/Los_Angeles");
const isDetecting = ref(false);
const isOperating = ref(false);

// 探测代理 IP 与推荐时区
const handleDetectProxy = async () => {
  isDetecting.value = true;
  try {
    const info = await invoke<ProxyGeoInfo>("detect_proxy_timezone");
    proxyInfo.value = info;
    // 默认自动将选定时区设为代理 IP 所在地的时区
    selectedTimezone.value = info.timezone_id;
  } catch (err: any) {
    alert("侦测代理时区失败: " + err);
  } finally {
    isDetecting.value = false;
  }
};

// 获取 ChatGPT 运行与时区状态
const fetchStatus = async () => {
  try {
    chatgptStatus.value = await invoke<ChatGPTStatus>("get_chatgpt_status");
  } catch (err) {
    console.error("获取 ChatGPT 状态失败:", err);
  }
};

// 获取常用预设
const loadPresets = async () => {
  try {
    presets.value = await invoke<TimezonePreset[]>("get_available_timezone_presets");
  } catch (err) {
    console.error("获取预设时区失败:", err);
  }
};

onMounted(async () => {
  await Promise.all([handleDetectProxy(), fetchStatus(), loadPresets()]);
});

// 以独立时区启动并注入 ChatGPT
const handleLaunchWithTimezone = async () => {
  if (!selectedTimezone.value) return;

  isOperating.value = true;
  try {
    const msg = await invoke<string>("launch_chatgpt_isolated_timezone", {
      timezoneId: selectedTimezone.value,
    });
    alert(msg);
    await fetchStatus();
  } catch (err: any) {
    alert("启动并注入失败: " + err);
  } finally {
    isOperating.value = false;
  }
};

// 热更新当前正在运行的 ChatGPT 时区
const handleInjectRunning = async () => {
  if (!selectedTimezone.value) return;
  isOperating.value = true;
  try {
    const count = await invoke<number>("inject_chatgpt_timezone_cdp", {
      timezoneId: selectedTimezone.value,
    });
    alert(`热更新成功！已向 ${count} 个渲染页面注入时区: ${selectedTimezone.value}`);
    await fetchStatus();
  } catch (err: any) {
    alert("热更新失败: " + err);
  } finally {
    isOperating.value = false;
  }
};

// 临时对齐系统时区
const handleAlignSystemTz = async () => {
  if (!proxyInfo.value?.windows_tz_name) {
    alert("尚未获取到代理对应的 Windows 时区名称");
    return;
  }
  try {
    await invoke("set_system_timezone_align", {
      windowsTz: proxyInfo.value.windows_tz_name,
    });
    alert(`系统时区已临时对齐为代理时区: ${proxyInfo.value.windows_tz_name}`);
    await fetchStatus();
  } catch (err: any) {
    alert("对齐系统时区失败: " + err);
  }
};

// 恢复系统原始时区
const handleRestoreSystemTz = async () => {
  try {
    const orig = await invoke<string>("restore_system_timezone");
    alert(`系统时区已恢复为: ${orig}`);
    await fetchStatus();
  } catch (err: any) {
    alert("恢复系统时区失败: " + err);
  }
};
</script>

<style scoped>
.timezone-manager-root {
  display: flex;
  flex-direction: column;
  height: 100%;
  padding: 16px;
  gap: 14px;
  overflow-y: auto;
}

/* 顶部代理雷达卡片 */
.proxy-radar-card {
  padding: 12px 16px;
  display: flex;
  flex-direction: column;
  gap: 10px;
  background: rgba(14, 18, 30, 0.95);
  border: 1px solid rgba(0, 240, 255, 0.3);
}

.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.neon-icon {
  color: var(--cyber-neon-cyan);
}

.card-title {
  font-size: 13px;
  font-weight: 700;
  color: #fff;
  letter-spacing: 0.5px;
}

.geo-display-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 10px;
}

.geo-pill {
  background: rgba(8, 11, 20, 0.85);
  border: 1px solid rgba(0, 240, 255, 0.12);
  border-radius: 4px;
  padding: 8px 10px;
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.highlight-pill {
  border-color: rgba(0, 240, 255, 0.45);
  background: rgba(0, 240, 255, 0.05);
}

.pill-label {
  font-size: 10px;
  font-weight: 700;
  color: #64748b;
  text-transform: uppercase;
}

.pill-val {
  font-size: 13px;
  font-weight: 700;
  color: #e2e8f0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.ip-val {
  font-family: 'Consolas', monospace;
  color: var(--cyber-neon-yellow);
}

.val-cyan {
  color: var(--cyber-neon-cyan);
}

.val-green {
  color: var(--cyber-neon-green);
}

/* 状态看板卡片 */
.status-board-card {
  padding: 12px 16px;
  display: flex;
  flex-direction: column;
  gap: 10px;
  background: rgba(12, 16, 28, 0.9);
  border: 1px solid rgba(0, 240, 255, 0.2);
}

.board-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.mini-btn {
  font-size: 11px;
  padding: 2px 8px;
}

.status-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 14px;
}

.status-col {
  background: rgba(6, 9, 16, 0.7);
  border: 1px solid rgba(0, 240, 255, 0.1);
  border-radius: 4px;
  padding: 10px 12px;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.col-title {
  font-size: 11px;
  font-weight: 700;
  color: #94a3b8;
}

.status-tag-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.tz-active-text {
  font-size: 14px;
  font-weight: 800;
  color: var(--cyber-neon-cyan);
  font-family: 'Consolas', monospace;
}

.sub-hint {
  font-size: 10px;
  color: #64748b;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* 控制操作区 */
.action-panel-card {
  padding: 14px 16px;
  display: flex;
  flex-direction: column;
  gap: 12px;
  background: rgba(14, 18, 30, 0.95);
  border: 1px solid rgba(0, 240, 255, 0.25);
}

.panel-header {
  display: flex;
  align-items: center;
  gap: 8px;
}

.presets-section {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.presets-title {
  font-size: 11px;
  font-weight: 700;
  color: #94a3b8;
}

.presets-list {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.preset-btn {
  background: rgba(10, 14, 25, 0.85);
  border: 1px solid rgba(0, 240, 255, 0.18);
  border-radius: 4px;
  padding: 5px 10px;
  font-size: 11px;
  color: #cbd5e1;
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 4px;
  transition: all 0.15s ease;
}

.preset-btn:hover {
  background: rgba(0, 240, 255, 0.12);
  border-color: var(--cyber-neon-cyan);
  color: #fff;
}

.preset-btn.active {
  background: rgba(0, 240, 255, 0.25);
  border-color: var(--cyber-neon-cyan);
  color: #fff;
  box-shadow: 0 0 8px rgba(0, 240, 255, 0.3);
  font-weight: 700;
}

.proxy-preset-btn {
  border-color: var(--cyber-neon-yellow);
  color: var(--cyber-neon-yellow);
}

.p-region {
  color: #64748b;
  font-size: 10px;
}

.custom-input-row {
  display: flex;
  align-items: center;
  gap: 12px;
}

.input-label {
  font-size: 12px;
  font-weight: 700;
  color: #cbd5e1;
  flex-shrink: 0;
}

.input-wrapper {
  flex: 1;
}

.actions-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding-top: 6px;
  border-top: 1px solid rgba(0, 240, 255, 0.1);
}

.action-left,
.action-right {
  display: flex;
  align-items: center;
  gap: 10px;
}

.launch-inject-btn {
  height: 36px;
  padding: 0 16px;
  font-size: 13px;
}

.spin-anim {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}
</style>
