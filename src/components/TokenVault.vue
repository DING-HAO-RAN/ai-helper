<template>
  <div class="token-vault-root">
    <!-- 顶部添加与加密录入卡片 -->
    <section class="add-token-card cyber-card">
      <div class="card-header">
        <div class="header-left">
          <KeyRound class="neon-icon" :size="16" />
          <span class="card-title">ENCRYPTED CREDENTIALS // 录入 GitHub 访问令牌</span>
          <span class="cyber-badge cyber-badge-cyan">DPAPI 加密保护</span>
        </div>
      </div>

      <div class="form-grid">
        <div class="input-group alias-group">
          <label class="input-label">别名 (Alias)</label>
          <input
            v-model="newAlias"
            type="text"
            class="cyber-input"
            placeholder="例如: default, work, personal..."
          />
        </div>

        <div class="input-group token-group">
          <label class="input-label">GitHub Token (PAT / 凭据)</label>
          <div class="token-input-wrapper">
            <input
              v-model="newToken"
              :type="showInputPlain ? 'text' : 'password'"
              class="cyber-input token-input"
              placeholder="ghp_... 或 github_pat_..."
            />
            <button
              class="icon-inline-btn"
              :title="showInputPlain ? '隐藏明文' : '显示明文'"
              @click="showInputPlain = !showInputPlain"
            >
              <EyeOff v-if="showInputPlain" :size="14" />
              <Eye v-else :size="14" />
            </button>
            <button
              class="icon-inline-btn"
              title="从剪贴板粘贴"
              @click="handlePasteToken"
            >
              <ClipboardPaste :size="14" />
            </button>
          </div>
        </div>

        <div class="input-group note-group">
          <label class="input-label">用途备注 (可选)</label>
          <input
            v-model="newNote"
            type="text"
            class="cyber-input"
            placeholder="例如: 私有代码仓库只读访问..."
          />
        </div>

        <div class="action-group">
          <label class="checkbox-label">
            <input v-model="newIsDefault" type="checkbox" class="cyber-checkbox" />
            <span>设为默认令牌</span>
          </label>
          <button
            class="cyber-btn cyber-btn-primary save-btn"
            :disabled="!newAlias.trim() || !newToken.trim()"
            @click="handleSaveToken"
          >
            <Lock :size="14" />
            加密入库
          </button>
        </div>
      </div>
    </section>

    <!-- 中部 CLI 快速调用指南卡片 -->
    <section class="cli-guide-card cyber-card">
      <div class="guide-header">
        <Terminal class="neon-icon" :size="15" />
        <span class="guide-title">AI 工具与命令行终端直接访问指南 (CLI)</span>
      </div>
      <div class="guide-content">
        <p class="guide-desc">
          终端脚本与任何 AI 工具可直接通过下方命令毫秒级静默读取当前默认的解密令牌：
        </p>
        <div class="guide-commands">
          <div class="cmd-pill">
            <span class="prompt">$</span>
            <span class="text">ai-helper token get</span>
            <button class="mini-copy" @click="copyCli('ai-helper token get')">复制</button>
          </div>
          <div class="cmd-pill">
            <span class="prompt">PS:</span>
            <span class="text">$env:GITHUB_TOKEN = (ai-helper token get)</span>
            <button class="mini-copy" @click="copyCli('$env:GITHUB_TOKEN = (ai-helper token get)')">复制</button>
          </div>
        </div>
      </div>
    </section>

    <!-- 下方长条列表区域 -->
    <section class="list-section">
      <div class="list-header">
        <span class="list-title">CREDENTIALS REPOSITORY // 已加密凭据列表</span>
        <span class="cyber-badge cyber-badge-cyan">{{ tokens.length }} 个令牌</span>
      </div>

      <div class="token-scroll-list">
        <div v-if="tokens.length === 0" class="empty-state cyber-card">
          <ShieldAlert :size="32" class="empty-icon" />
          <p>本地暂未保存任何 GitHub 令牌。请在上方输入并加密保存！</p>
        </div>

        <div
          v-for="item in tokens"
          :key="item.id"
          class="token-row-card cyber-card"
        >
          <div class="row-left-bar" :class="{ 'is-default': item.is_default }"></div>

          <!-- 令牌信息主体 -->
          <div class="token-info-main">
            <div class="info-top-line">
              <span class="token-alias">{{ item.alias }}</span>
              <span v-if="item.is_default" class="cyber-badge cyber-badge-green">
                ★ 默认活动令牌
              </span>
              <span v-if="item.note" class="token-note">{{ item.note }}</span>
              <span class="update-time">{{ item.updated_at }}</span>
            </div>

            <!-- 密文显示与显隐切换 -->
            <div class="token-secret-line">
              <span class="secret-label">TOKEN:</span>
              <code class="secret-code">
                {{ revealedTokens[item.alias] || item.masked_token }}
              </code>
              <button
                class="reveal-toggle-btn"
                :title="revealedTokens[item.alias] ? '隐藏明文' : '显示明文'"
                @click="toggleReveal(item.alias)"
              >
                <EyeOff v-if="revealedTokens[item.alias]" :size="13" />
                <Eye v-else :size="13" />
              </button>
            </div>

            <!-- 连通性测试结果提示区 -->
            <div v-if="testResults[item.id]" class="test-result-box" :class="testResults[item.id].success ? 'res-success' : 'res-fail'">
              <CheckCircle v-if="testResults[item.id].success" :size="13" />
              <AlertCircle v-else :size="13" />
              <span>{{ testResults[item.id].message }}</span>
            </div>
          </div>

          <!-- 右侧操作栏 -->
          <div class="token-actions">
            <!-- 测试连通性按钮 -->
            <button
              class="cyber-btn"
              :disabled="testingId === item.id"
              title="在线请求 GitHub API 验证令牌有效性"
              @click="handleTestConnection(item)"
            >
              <Activity :size="14" :class="{ 'spin-anim': testingId === item.id }" />
              <span>{{ testingId === item.id ? "验证中..." : "测试连接" }}</span>
            </button>

            <!-- 复制明文 -->
            <button
              class="cyber-btn"
              :class="{ 'copied-success': copiedAlias === item.alias }"
              title="解密并复制明文 Token 至剪贴板"
              @click="handleCopySecret(item.alias)"
            >
              <Check v-if="copiedAlias === item.alias" :size="14" />
              <Copy v-else :size="14" />
              <span>{{ copiedAlias === item.alias ? "已复制" : "复制明文" }}</span>
            </button>

            <!-- 设为默认 -->
            <button
              v-if="!item.is_default"
              class="cyber-btn"
              title="将此令牌设为命令行与默认调用项"
              @click="handleSetDefault(item.id)"
            >
              <Star :size="14" />
              <span>设为默认</span>
            </button>

            <!-- 删除 -->
            <button
              class="cyber-btn cyber-btn-danger"
              title="删除此凭据"
              @click="handleDeleteToken(item)"
            >
              <Trash2 :size="14" />
              <span>删除</span>
            </button>
          </div>
        </div>
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import {
  KeyRound,
  Eye,
  EyeOff,
  ClipboardPaste,
  Lock,
  Terminal,
  ShieldAlert,
  Activity,
  Copy,
  Star,
  Trash2,
  Check,
  CheckCircle,
  AlertCircle,
} from "lucide-vue-next";
import type { TokenDisplayView, GitHubUserInfo } from "../types/token";

const tokens = ref<TokenDisplayView[]>([]);
const newAlias = ref("default");
const newToken = ref("");
const newNote = ref("");
const newIsDefault = ref(false);
const showInputPlain = ref(false);

const revealedTokens = ref<Record<string, string>>({});
const testingId = ref<string | null>(null);
const testResults = ref<Record<string, { success: boolean; message: string }>>({});
const copiedAlias = ref<string | null>(null);

// 加载凭据列表
const loadTokens = async () => {
  try {
    tokens.value = await invoke<TokenDisplayView[]>("get_tokens");
  } catch (err) {
    console.error("加载令牌失败:", err);
  }
};

onMounted(() => {
  loadTokens();
});

// 从剪贴板粘贴
const handlePasteToken = async () => {
  try {
    const text = await navigator.clipboard.readText();
    if (text) newToken.value = text.trim();
  } catch {
    alert("无法读取剪贴板，请直接粘贴");
  }
};

// 保存令牌
const handleSaveToken = async () => {
  if (!newAlias.value.trim() || !newToken.value.trim()) return;

  try {
    await invoke("save_token", {
      alias: newAlias.value.trim(),
      plainToken: newToken.value.trim(),
      note: newNote.value.trim(),
      isDefault: newIsDefault.value,
    });

    newToken.value = "";
    newNote.value = "";
    newAlias.value = "default";
    newIsDefault.value = false;
    await loadTokens();
  } catch (err) {
    alert("保存令牌失败: " + err);
  }
};

// 切换显示明文
const toggleReveal = async (alias: string) => {
  if (revealedTokens.value[alias]) {
    delete revealedTokens.value[alias];
  } else {
    try {
      const plain = await invoke<string>("get_token_plain_text", { alias });
      revealedTokens.value[alias] = plain;
    } catch (err) {
      alert("解密失败: " + err);
    }
  }
};

// 复制明文
const handleCopySecret = async (alias: string) => {
  try {
    const plain = await invoke<string>("get_token_plain_text", { alias });
    await navigator.clipboard.writeText(plain);
    copiedAlias.value = alias;
    setTimeout(() => {
      if (copiedAlias.value === alias) copiedAlias.value = null;
    }, 2000);
  } catch (err) {
    alert("复制失败: " + err);
  }
};

// 设为默认
const handleSetDefault = async (id: string) => {
  try {
    await invoke("set_default_token", { id });
    await loadTokens();
  } catch (err) {
    alert("设置默认失败: " + err);
  }
};

// 删除令牌
const handleDeleteToken = async (item: TokenDisplayView) => {
  if (confirm(`确定要从本地安全保险库中删除令牌 "${item.alias}" 吗？`)) {
    try {
      await invoke("delete_token", { id: item.id });
      delete revealedTokens.value[item.alias];
      delete testResults.value[item.id];
      await loadTokens();
    } catch (err) {
      alert("删除失败: " + err);
    }
  }
};

// 测试 GitHub 连通性
const handleTestConnection = async (item: TokenDisplayView) => {
  testingId.value = item.id;
  try {
    const plain = await invoke<string>("get_token_plain_text", { alias: item.alias });
    const user = await invoke<GitHubUserInfo>("test_github_token", { token: plain });
    testResults.value[item.id] = {
      success: true,
      message: `连通成功! 用户: ${user.login} (${user.name || "无昵称"}) | 公开仓库数: ${user.public_repos || 0}`,
    };
  } catch (err: any) {
    testResults.value[item.id] = {
      success: false,
      message: `验证失败: ${err.toString()}`,
    };
  } finally {
    testingId.value = null;
  }
};

// 复制 CLI 命令
const copyCli = (cmd: string) => {
  navigator.clipboard.writeText(cmd);
  alert("命令已复制至剪贴板");
};
</script>

<style scoped>
.token-vault-root {
  display: flex;
  flex-direction: column;
  height: 100%;
  padding: 16px;
  gap: 14px;
  overflow: hidden;
}

/* 录入卡片 */
.add-token-card {
  padding: 12px 14px;
  display: flex;
  flex-direction: column;
  gap: 10px;
  flex-shrink: 0;
  background: rgba(14, 18, 30, 0.95);
  border: 1px solid rgba(0, 240, 255, 0.25);
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

.form-grid {
  display: grid;
  grid-template-columns: 140px 1fr 180px auto;
  gap: 10px;
  align-items: flex-end;
}

.input-group {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.input-label {
  font-size: 11px;
  font-weight: 600;
  color: #94a3b8;
}

.token-input-wrapper {
  position: relative;
  display: flex;
  align-items: center;
}

.token-input {
  padding-right: 60px;
  font-family: 'Consolas', monospace;
}

.icon-inline-btn {
  position: absolute;
  right: 6px;
  background: transparent;
  border: none;
  color: #64748b;
  cursor: pointer;
  display: flex;
  align-items: center;
  padding: 4px;
}

.icon-inline-btn:first-of-type {
  right: 32px;
}

.icon-inline-btn:hover {
  color: var(--cyber-neon-cyan);
}

.action-group {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 6px;
}

.checkbox-label {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 11px;
  color: #cbd5e1;
  cursor: pointer;
}

.cyber-checkbox {
  accent-color: var(--cyber-neon-cyan);
  cursor: pointer;
}

.save-btn {
  height: 34px;
}

/* CLI 指南卡片 */
.cli-guide-card {
  padding: 10px 14px;
  background: rgba(10, 13, 24, 0.9);
  border: 1px solid rgba(0, 240, 255, 0.15);
  display: flex;
  flex-direction: column;
  gap: 6px;
  flex-shrink: 0;
}

.guide-header {
  display: flex;
  align-items: center;
  gap: 8px;
}

.guide-title {
  font-size: 12px;
  font-weight: 700;
  color: var(--cyber-neon-cyan);
  letter-spacing: 0.5px;
}

.guide-content {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.guide-desc {
  font-size: 11px;
  color: #94a3b8;
}

.guide-commands {
  display: flex;
  gap: 8px;
}

.cmd-pill {
  display: flex;
  align-items: center;
  gap: 6px;
  background: rgba(6, 9, 18, 0.9);
  border: 1px solid rgba(0, 240, 255, 0.2);
  border-radius: 4px;
  padding: 3px 8px;
  font-family: 'Consolas', monospace;
  font-size: 11px;
}

.cmd-pill .prompt {
  color: var(--cyber-neon-pink);
  font-weight: 700;
}

.cmd-pill .text {
  color: #e2e8f0;
}

.mini-copy {
  background: rgba(0, 240, 255, 0.1);
  border: 1px solid rgba(0, 240, 255, 0.3);
  color: var(--cyber-neon-cyan);
  border-radius: 2px;
  padding: 1px 4px;
  font-size: 10px;
  cursor: pointer;
}

.mini-copy:hover {
  background: var(--cyber-neon-cyan);
  color: #000;
}

/* 列表区域 */
.list-section {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
  gap: 8px;
}

.list-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.list-title {
  font-size: 12px;
  font-weight: 700;
  color: #94a3b8;
}

.token-scroll-list {
  flex: 1;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding-right: 4px;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 40px 20px;
  color: #64748b;
  gap: 10px;
  font-size: 13px;
}

.empty-icon {
  color: rgba(0, 240, 255, 0.3);
}

.token-row-card {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 14px;
  gap: 14px;
  background: rgba(14, 18, 30, 0.85);
  border: 1px solid rgba(0, 240, 255, 0.12);
  position: relative;
  transition: all 0.2s ease;
}

.token-row-card:hover {
  background: rgba(18, 23, 38, 0.95);
  border-color: rgba(0, 240, 255, 0.4);
}

.row-left-bar {
  position: absolute;
  left: 0;
  top: 15%;
  bottom: 15%;
  width: 3px;
  background: rgba(0, 240, 255, 0.3);
  border-radius: 2px;
}

.row-left-bar.is-default {
  background: var(--cyber-neon-green);
  box-shadow: 0 0 6px var(--cyber-neon-green);
}

.token-info-main {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.info-top-line {
  display: flex;
  align-items: center;
  gap: 8px;
}

.token-alias {
  font-size: 13px;
  font-weight: 700;
  color: #fff;
}

.token-note {
  font-size: 11px;
  color: #94a3b8;
  background: rgba(255, 255, 255, 0.05);
  padding: 1px 6px;
  border-radius: 3px;
}

.update-time {
  font-size: 10px;
  color: #64748b;
  margin-left: auto;
}

.token-secret-line {
  display: flex;
  align-items: center;
  gap: 8px;
}

.secret-label {
  font-size: 11px;
  font-weight: 700;
  color: #64748b;
}

.secret-code {
  font-size: 12px;
  font-family: 'Consolas', monospace;
  color: var(--cyber-neon-cyan);
  background: rgba(0, 0, 0, 0.4);
  padding: 2px 6px;
  border-radius: 3px;
  border: 1px solid rgba(0, 240, 255, 0.1);
}

.reveal-toggle-btn {
  background: transparent;
  border: none;
  color: #64748b;
  cursor: pointer;
  display: flex;
  align-items: center;
}

.reveal-toggle-btn:hover {
  color: var(--cyber-neon-cyan);
}

.test-result-box {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 11px;
  margin-top: 2px;
  padding: 2px 6px;
  border-radius: 3px;
}

.res-success {
  background: rgba(0, 255, 157, 0.1);
  color: var(--cyber-neon-green);
  border: 1px solid rgba(0, 255, 157, 0.3);
}

.res-fail {
  background: rgba(255, 0, 85, 0.1);
  color: var(--cyber-neon-pink);
  border: 1px solid rgba(255, 0, 85, 0.3);
}

.token-actions {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
}

.copied-success {
  background: rgba(0, 255, 157, 0.25) !important;
  border-color: var(--cyber-neon-green) !important;
  color: #fff !important;
}

.spin-anim {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}
</style>
