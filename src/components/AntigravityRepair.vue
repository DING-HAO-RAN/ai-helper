<template>
  <div class="antigravity-repair-root">
    <!-- 顶部：核心诊断健康体检看板 -->
    <section class="diagnostic-card cyber-card">
      <div class="card-header">
        <div class="header-left">
          <Wrench class="neon-icon" :size="18" />
          <span class="card-title">ANTIGRAVITY PROXY DOCTOR // 代理体检与诊断</span>
          <span v-if="diagInfo?.installed" class="cyber-badge cyber-badge-green">已检测到安装</span>
          <span v-else class="cyber-badge cyber-badge-pink">未检测到安装</span>
        </div>
        <button class="cyber-btn mini-btn" :disabled="loading" @click="runDiagnosis">
          <RefreshCw :size="13" :class="{ 'spin-anim': loading }" />
          <span>重新体检</span>
        </button>
      </div>

      <div class="diag-grid">
        <!-- Antigravity 进程与安装 -->
        <div class="diag-col">
          <div class="col-header">
            <span class="col-title">客户端状态</span>
            <span
              class="cyber-badge"
              :class="diagInfo?.is_running ? 'cyber-badge-green' : 'cyber-badge-yellow'"
            >
              {{ diagInfo?.is_running ? `● 运行中 (${diagInfo.pids.length} 进程)` : "○ 未运行" }}
            </span>
          </div>
          <div class="info-row">
            <span class="label">程序路径:</span>
            <code class="val-code" :title="diagInfo?.exe_path || '未知'">
              {{ diagInfo?.exe_path || "未检测到 Antigravity.exe" }}
            </code>
          </div>
          <div class="info-row">
            <span class="label">快捷方式:</span>
            <span
              class="cyber-badge"
              :class="diagInfo?.shortcut_has_proxy_arg ? 'cyber-badge-green' : 'cyber-badge-yellow'"
            >
              {{ diagInfo?.shortcut_has_proxy_arg ? "✓ 已附带代理参数" : "! 缺少 --proxy-server 参数" }}
            </span>
          </div>
        </div>

        <!-- 本地活跃代理侦测 -->
        <div class="diag-col">
          <div class="col-header">
            <span class="col-title">本地网络代理</span>
            <span
              class="cyber-badge"
              :class="diagInfo?.detected_local_proxy ? 'cyber-badge-cyan' : 'cyber-badge-yellow'"
            >
              {{ diagInfo?.detected_local_proxy ? "● 活跃监听中" : "○ 未检测到活跃代理" }}
            </span>
          </div>
          <div class="info-row">
            <span class="label">侦测端口:</span>
            <span class="val-bright">{{ diagInfo?.detected_local_proxy || "未识别 (可手动指定)" }}</span>
          </div>
          <div class="info-row">
            <span class="label">系统代理:</span>
            <span
              class="cyber-badge"
              :class="diagInfo?.system_proxy_enable ? 'cyber-badge-green' : 'cyber-badge-pink'"
            >
              {{ diagInfo?.system_proxy_enable ? "✓ 已开启" : "✗ 未开启 (ProxyEnable=0)" }}
            </span>
          </div>
        </div>

        <!-- 环境变量状态 -->
        <div class="diag-col">
          <div class="col-header">
            <span class="col-title">全局环境代理变量 (Go/Node 必需)</span>
            <span
              class="cyber-badge"
              :class="hasEnvProxy ? 'cyber-badge-green' : 'cyber-badge-pink'"
            >
              {{ hasEnvProxy ? "✓ 已配置生效" : "✗ 缺失 (导致直连超时)" }}
            </span>
          </div>
          <div class="env-status-list">
            <div class="env-item">
              <span class="env-k">HTTP_PROXY:</span>
              <span class="env-v" :class="{ 'v-none': !diagInfo?.env_http_proxy }">
                {{ diagInfo?.env_http_proxy || "未配置 (直连)" }}
              </span>
            </div>
            <div class="env-item">
              <span class="env-k">HTTPS_PROXY:</span>
              <span class="env-v" :class="{ 'v-none': !diagInfo?.env_https_proxy }">
                {{ diagInfo?.env_https_proxy || "未配置 (直连)" }}
              </span>
            </div>
          </div>
        </div>
      </div>
    </section>

    <!-- 中部：故障原因机理深度解析 -->
    <section class="reason-card cyber-card">
      <div class="reason-header">
        <AlertCircle class="neon-icon" :size="15" />
        <span class="reason-title">为什么 Antigravity 默认无法走系统代理？</span>
      </div>
      <p class="reason-desc">
        Antigravity 内部采用 <b>Golang (Go语言)</b> 与 <b>Node.js</b> 核心守护进程处理 AI 通信与 Google CloudCode 服务。
        在 Windows 操作系统中，<b>Go 语言和 Node.js 严格忽略 Windows 系统代理设置（WinINet）</b>。一键修复会写入大小写代理环境变量并启用 <code>NODE_USE_ENV_PROXY</code>，使 Antigravity 及其启动的 DeepSeek Harness 等 Node 插件统一继承代理。
      </p>
    </section>

    <!-- 下部：一键自动修复与快捷操作区 -->
    <section class="action-card cyber-card">
      <div class="card-header">
        <div class="header-left">
          <Zap class="neon-icon" :size="17" />
          <span class="card-title">ONE-CLICK PROXY FIX // 一键全自动修复中心</span>
        </div>
      </div>

      <!-- 目标代理配置行 -->
      <div class="proxy-input-row">
        <label class="input-label">修复目标代理地址:</label>
        <div class="input-wrapper">
          <input
            v-model="targetProxy"
            type="text"
            class="cyber-input"
            placeholder="例如: http://127.0.0.1:7897 或 http://127.0.0.1:7890"
          />
        </div>
        <button
          v-if="diagInfo?.detected_local_proxy && targetProxy !== diagInfo.detected_local_proxy"
          class="cyber-btn"
          @click="targetProxy = diagInfo.detected_local_proxy"
        >
          填入检测到的代理
        </button>
      </div>

      <!-- 核心操作按钮栏 -->
      <div class="action-btn-toolbar">
        <div class="btn-group-left">
          <button
            class="cyber-btn cyber-btn-primary big-fix-btn"
            :disabled="isFixing || !targetProxy"
            title="自动写入用户全局环境变量、修复桌面快捷方式参数并开启系统代理注册表"
            @click="handleApplyFix"
          >
            <Sparkles :size="16" />
            <span>{{ isFixing ? "正在执行全自动修复..." : "⚡ 一键全面修复 Antigravity 代理 (推荐)" }}</span>
          </button>

          <button
            class="cyber-btn"
            :disabled="isTesting"
            title="在线测试能否通过代理连接到 Google CloudCode / Gemini 官方服务器"
            @click="handleTestGoogleApi"
          >
            <Activity :size="14" :class="{ 'spin-anim': isTesting }" />
            <span>{{ isTesting ? "正在测试连通性..." : "测试 Google API 连通性" }}</span>
          </button>
        </div>

        <div class="btn-group-right">
          <button
            class="cyber-btn cyber-btn-success"
            title="以当前代理环境变量及参数直接启动 Antigravity"
            @click="handleLaunchAntigravity"
          >
            <Power :size="14" />
            <span>以强制代理拉起 Antigravity</span>
          </button>

          <button
            class="cyber-btn cyber-btn-danger"
            title="从用户注册表中清除 HTTP_PROXY / HTTPS_PROXY 并还原快捷方式"
            @click="handleClearProxy"
          >
            <RotateCcw :size="14" />
            <span>清除 / 还原代理设置</span>
          </button>
        </div>
      </div>

      <!-- 连通性测试结果面板 -->
      <div v-if="testResult" class="test-res-banner" :class="testResult.success ? 'banner-pass' : 'banner-fail'">
        <CheckCircle2 v-if="testResult.success" :size="16" />
        <AlertTriangle v-else :size="16" />
        <div class="test-res-text">
          <span class="res-status">
            {{ testResult.success ? "✓ Google Gemini API 连通成功！代理已生效！" : "✗ Google API 校验失败" }}
          </span>
          <span class="res-details">
            目标: {{ testResult.target_url }} | 耗时: {{ testResult.latency_ms }} ms | 响应码: {{ testResult.status_code }}
            {{ testResult.error_msg ? ` | 错误: ${testResult.error_msg}` : "" }}
          </span>
        </div>
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import {
  Wrench,
  RefreshCw,
  AlertCircle,
  Zap,
  Sparkles,
  Activity,
  Power,
  RotateCcw,
  CheckCircle2,
  AlertTriangle,
} from "lucide-vue-next";
import { cyberAlert, cyberConfirm, cyberToast } from "../utils/dialog";

interface AntigravityDiagnostic {
  installed: boolean;
  exe_path: string | null;
  is_running: boolean;
  pids: number[];
  detected_local_proxy: string | null;
  env_http_proxy: string | null;
  env_https_proxy: string | null;
  env_all_proxy: string | null;
  system_proxy_enable: boolean;
  system_proxy_server: string | null;
  desktop_shortcut_found: boolean;
  shortcut_has_proxy_arg: boolean;
  shortcut_path: string | null;
}

interface AntigravityFixResult {
  success: boolean;
  proxy_applied: string;
  env_fixed: boolean;
  shortcut_fixed: boolean;
  system_proxy_fixed: boolean;
  broadcast_sent: boolean;
  message: string;
}

interface GoogleApiTestResult {
  success: boolean;
  latency_ms: number;
  status_code: number;
  target_url: string;
  error_msg: string | null;
}

const diagInfo = ref<AntigravityDiagnostic | null>(null);
const targetProxy = ref("http://127.0.0.1:7897");
const loading = ref(false);
const isFixing = ref(false);
const isTesting = ref(false);
const testResult = ref<GoogleApiTestResult | null>(null);

const hasEnvProxy = computed(() => {
  return !!diagInfo.value?.env_http_proxy && !!diagInfo.value?.env_https_proxy;
});

// 执行体检诊断
const runDiagnosis = async () => {
  loading.value = true;
  try {
    const res = await invoke<AntigravityDiagnostic>("diagnose_antigravity_status");
    diagInfo.value = res;
    if (res.detected_local_proxy) {
      targetProxy.value = res.detected_local_proxy;
    }
  } catch (err) {
    console.error("诊断失败:", err);
  } finally {
    loading.value = false;
  }
};

onMounted(() => {
  runDiagnosis();
});

// 一键全面修复
const handleApplyFix = async () => {
  if (!targetProxy.value.trim()) return;

  isFixing.value = true;
  try {
    const res = await invoke<AntigravityFixResult>("fix_antigravity_proxy_action", {
      customProxy: targetProxy.value.trim(),
    });
    await cyberAlert(res.message, "代理修复完成", "success");
    await runDiagnosis();
  } catch (err: any) {
    cyberAlert("修复失败: " + err, "错误", "error");
  } finally {
    isFixing.value = false;
  }
};

// 测试 Google API 连通性
const handleTestGoogleApi = async () => {
  isTesting.value = true;
  testResult.value = null;
  try {
    const res = await invoke<GoogleApiTestResult>("test_antigravity_google_api", {
      customProxy: targetProxy.value.trim() || null,
    });
    testResult.value = res;
    if (res.success) {
      cyberToast(`Google API 连通成功 (${res.latency_ms}ms)`, "success");
    } else {
      cyberToast("Google API 连通失败", "error");
    }
  } catch (err: any) {
    cyberAlert("测试失败: " + err, "测试错误", "error");
  } finally {
    isTesting.value = false;
  }
};

// 强制代理启动 Antigravity
const handleLaunchAntigravity = async () => {
  try {
    const msg = await invoke<string>("launch_antigravity_action", {
      customProxy: targetProxy.value.trim() || null,
    });
    cyberToast(msg, "success");
    setTimeout(runDiagnosis, 1500);
  } catch (err: any) {
    cyberAlert("启动失败: " + err, "错误", "error");
  }
};

// 一键清除还原
const handleClearProxy = async () => {
  const confirmed = await cyberConfirm(
    "确定要清除系统用户环境变量中的 HTTP_PROXY / HTTPS_PROXY 并还原快捷方式参数吗？",
    "还原代理设置确认"
  );
  if (confirmed) {
    try {
      const msg = await invoke<string>("clear_antigravity_proxy_action");
      cyberToast(msg, "info");
      await runDiagnosis();
    } catch (err: any) {
      cyberAlert("清除失败: " + err, "错误", "error");
    }
  }
};
</script>

<style scoped>
.antigravity-repair-root {
  display: flex;
  flex-direction: column;
  height: 100%;
  padding: 16px;
  gap: 14px;
  overflow-y: auto;
}

/* 顶部体检卡片 */
.diagnostic-card {
  padding: 14px 16px;
  display: flex;
  flex-direction: column;
  gap: 12px;
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

.mini-btn {
  font-size: 11px;
  padding: 3px 8px;
}

.diag-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 12px;
}

.diag-col {
  background: rgba(8, 11, 20, 0.85);
  border: 1px solid rgba(0, 240, 255, 0.12);
  border-radius: 4px;
  padding: 10px 12px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.col-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.col-title {
  font-size: 11px;
  font-weight: 700;
  color: #94a3b8;
}

.info-row {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 11px;
}

.label {
  color: #64748b;
  flex-shrink: 0;
}

.val-code {
  color: #cbd5e1;
  font-family: 'Consolas', monospace;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.val-bright {
  color: var(--cyber-neon-cyan);
  font-weight: 700;
  font-family: 'Consolas', monospace;
}

.env-status-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.env-item {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 11px;
  font-family: 'Consolas', monospace;
}

.env-k {
  color: #64748b;
  width: 95px;
}

.env-v {
  color: var(--cyber-neon-green);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.env-v.v-none {
  color: var(--cyber-neon-pink);
}

/* 机理解析卡片 */
.reason-card {
  padding: 10px 16px;
  background: rgba(10, 13, 22, 0.9);
  border: 1px solid rgba(252, 238, 10, 0.25);
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.reason-header {
  display: flex;
  align-items: center;
  gap: 8px;
}

.reason-title {
  font-size: 12px;
  font-weight: 700;
  color: var(--cyber-neon-yellow);
}

.reason-desc {
  font-size: 11px;
  color: #94a3b8;
  line-height: 1.6;
}

.reason-desc code {
  color: var(--cyber-neon-cyan);
  background: rgba(0, 0, 0, 0.4);
  padding: 1px 4px;
  border-radius: 3px;
}

/* 操作卡片 */
.action-card {
  padding: 14px 16px;
  display: flex;
  flex-direction: column;
  gap: 12px;
  background: rgba(14, 18, 30, 0.95);
  border: 1px solid rgba(0, 240, 255, 0.25);
}

.proxy-input-row {
  display: flex;
  align-items: center;
  gap: 10px;
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

.action-btn-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding-top: 6px;
  border-top: 1px solid rgba(0, 240, 255, 0.1);
}

.btn-group-left,
.btn-group-right {
  display: flex;
  align-items: center;
  gap: 10px;
}

.big-fix-btn {
  height: 36px;
  padding: 0 18px;
  font-size: 13px;
}

.test-res-banner {
  padding: 10px 14px;
  border-radius: 4px;
  display: flex;
  align-items: flex-start;
  gap: 10px;
  font-size: 12px;
}

.banner-pass {
  background: rgba(0, 255, 157, 0.12);
  border: 1px solid rgba(0, 255, 157, 0.35);
  color: var(--cyber-neon-green);
}

.banner-fail {
  background: rgba(255, 0, 85, 0.12);
  border: 1px solid rgba(255, 0, 85, 0.35);
  color: var(--cyber-neon-pink);
}

.test-res-text {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.res-status {
  font-weight: 700;
}

.res-details {
  font-size: 11px;
  color: #cbd5e1;
}

.spin-anim {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}
</style>
