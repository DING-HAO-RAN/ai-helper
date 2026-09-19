<template>
  <div class="agent-manager-root">
    <!-- 顶部扫描控制与全局批量注入栏 -->
    <section class="scan-header-card cyber-card">
      <div class="header-left">
        <div class="title-row">
          <Bot class="neon-icon" :size="18" />
          <span class="header-title">AGENT.md MATRIX // 全盘 AI 工具与项目扫描</span>
        </div>
        <div class="stats-row">
          <div class="stat-pill">
            <span class="label">已索引文件:</span>
            <span class="val">{{ agents.length }}</span>
          </div>
          <div class="stat-pill">
            <span class="label">已注入令牌指引:</span>
            <span class="val-green">{{ configuredCount }}</span>
          </div>
          <div class="stat-pill">
            <span class="label">未注入:</span>
            <span class="val-yellow">{{ unconfiguredCount }}</span>
          </div>
          <div v-if="indexUpdatedAt" class="stat-pill index-time-pill">
            <span class="cyber-badge cyber-badge-green">
              ⚡ 本地索引生效中 ({{ indexUpdatedAt }})
            </span>
          </div>
        </div>
      </div>

      <div class="header-actions">
        <button
          class="cyber-btn"
          :disabled="isScanning"
          title="全盘重新扫描驱动器并更新程序所在目录的 agent_index.json"
          @click="handleRefreshScan"
        >
          <Radar :size="15" :class="{ 'spin-anim': isScanning }" />
          <span>{{ isScanning ? "全盘扫描中..." : "全盘扫描 / 更新索引" }}</span>
        </button>

        <button
          class="cyber-btn cyber-btn-primary inject-all-btn"
          :disabled="agents.length === 0 || isInjecting"
          title="一键将本地 GitHub 访问凭据调用方法写入所有已扫描的 AGENT.md"
          @click="handleInjectAll"
        >
          <Sparkles :size="15" />
          <span>{{ isInjecting ? "写入中..." : "一键写入所有 AGENT.md" }}</span>
        </button>
      </div>
    </section>

    <!-- 列表搜索与筛选工具栏 -->
    <div class="list-toolbar">
      <div class="toolbar-left">
        <span class="list-title">AGENT.md REPOSITORY // 识别列表</span>
        <span class="cyber-badge cyber-badge-cyan">{{ filteredAgents.length }} 项匹配</span>
      </div>

      <div class="toolbar-right">
        <!-- 状态快速筛选 -->
        <div class="filter-tabs">
          <button
            class="tab-btn"
            :class="{ active: filterType === 'all' }"
            @click="filterType = 'all'"
          >
            全部
          </button>
          <button
            class="tab-btn"
            :class="{ active: filterType === 'unconfigured' }"
            @click="filterType = 'unconfigured'"
          >
            待注入 ({{ unconfiguredCount }})
          </button>
          <button
            class="tab-btn"
            :class="{ active: filterType === 'configured' }"
            @click="filterType = 'configured'"
          >
            已注入 ({{ configuredCount }})
          </button>
        </div>

        <div class="search-input-box">
          <Search :size="14" class="search-icon" />
          <input
            v-model="searchKeyword"
            type="text"
            class="cyber-input search-input"
            placeholder="搜索路径、AI工具或项目名称..."
          />
        </div>
      </div>
    </div>

    <!-- 中下方长条卡片列表（垂直滚动） -->
    <div class="agent-scroll-list">
      <div v-if="filteredAgents.length === 0" class="empty-state cyber-card">
        <FileSearch :size="36" class="empty-icon" />
        <p v-if="agents.length === 0">
          尚未执行扫描。点击右上角【全盘智能扫描】按钮，秒级检索全电脑 AI 工具与项目 agent.md！
        </p>
        <p v-else>无符合当前筛选条件的文件。</p>
      </div>

      <div
        v-for="item in filteredAgents"
        :key="item.path"
        class="agent-row-card cyber-card"
      >
        <div
          class="row-left-bar"
          :class="item.has_token_guide ? 'bar-green' : 'bar-yellow'"
        ></div>

        <!-- 卡片主体内容 -->
        <div class="agent-info-main">
          <div class="info-top-line">
            <span class="cyber-badge cyber-badge-cyan tool-badge">
              {{ item.tool_or_project }}
            </span>
            <span class="file-name">{{ item.filename }}</span>
            <span
              class="cyber-badge"
              :class="item.has_token_guide ? 'cyber-badge-green' : 'cyber-badge-yellow'"
            >
              {{ item.has_token_guide ? "✓ 已配置令牌指引" : "! 未配置指引" }}
            </span>
            <span class="file-size">{{ formatBytes(item.size_bytes) }}</span>
            <span class="update-time">{{ item.modified_at }}</span>
          </div>

          <div class="file-path-line" :title="item.path">
            <Folder :size="12" class="path-icon" />
            <span class="path-text">{{ item.path }}</span>
          </div>

          <div class="file-preview-line" :title="item.preview">
            {{ item.preview }}
          </div>
        </div>

        <!-- 右侧操作三联按钮 -->
        <div class="agent-actions">
          <!-- 在资源管理器中打开定位 -->
          <button
            class="cyber-btn"
            title="在系统资源管理器中打开所在目录"
            @click="handleOpenFolder(item.path)"
          >
            <ExternalLink :size="14" />
            <span>定位目录</span>
          </button>

          <!-- 查看与编辑全文 -->
          <button
            class="cyber-btn"
            title="查看或直接编辑此 agent.md 内容"
            @click="handleViewEdit(item)"
          >
            <FileText :size="14" />
            <span>查看/编辑</span>
          </button>

          <!-- 单个文件一键写入 -->
          <button
            class="cyber-btn"
            :class="item.has_token_guide ? 'cyber-btn-success' : 'cyber-btn-primary'"
            title="将 GitHub 令牌指引写入此文件"
            @click="handleInjectSingle(item)"
          >
            <Check v-if="item.has_token_guide" :size="14" />
            <FileCode v-else :size="14" />
            <span>{{ item.has_token_guide ? "重新写入" : "写入指引" }}</span>
          </button>
        </div>
      </div>
    </div>

    <!-- 查看与编辑 Markdown 文件弹窗 -->
    <div v-if="activeEditItem" class="modal-overlay" @click.self="activeEditItem = null">
      <div class="cyber-modal cyber-card">
        <div class="modal-header">
          <div class="header-left">
            <FileCode :size="16" class="neon-icon" />
            <span class="modal-title">AGENT.md EDITOR // {{ activeEditItem.filename }}</span>
          </div>
          <button class="close-btn" @click="activeEditItem = null">
            <X :size="16" />
          </button>
        </div>

        <div class="modal-path-bar">
          <code>{{ activeEditItem.path }}</code>
        </div>

        <div class="modal-editor-body">
          <textarea
            v-model="editFileContent"
            class="cyber-input modal-textarea"
            placeholder="文件内容加载中..."
          ></textarea>
        </div>

        <div class="modal-footer">
          <button
            class="cyber-btn"
            @click="handleInjectInModal"
          >
            <Sparkles :size="14" />
            追加/更新令牌指引
          </button>
          <div class="footer-right">
            <button class="cyber-btn" @click="activeEditItem = null">
              取消
            </button>
            <button class="cyber-btn cyber-btn-primary" @click="handleSaveFileContent">
              <Save :size="14" />
              保存文件
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import {
  Bot,
  Radar,
  Sparkles,
  Search,
  FileSearch,
  Folder,
  ExternalLink,
  FileText,
  FileCode,
  Check,
  X,
  Save,
} from "lucide-vue-next";
import type { AgentFileInfo, AgentIndexCache, InjectSummary } from "../types/agent";

const agents = ref<AgentFileInfo[]>([]);
const indexUpdatedAt = ref("");
const isScanning = ref(false);
const isInjecting = ref(false);
const searchKeyword = ref("");
const filterType = ref<"all" | "configured" | "unconfigured">("all");

// 弹窗编辑
const activeEditItem = ref<AgentFileInfo | null>(null);
const editFileContent = ref("");

// 统计
const configuredCount = computed(
  () => agents.value.filter((a) => a.has_token_guide).length
);
const unconfiguredCount = computed(
  () => agents.value.filter((a) => !a.has_token_guide).length
);

// 过滤列表
const filteredAgents = computed(() => {
  let list = agents.value;
  if (filterType.value === "configured") {
    list = list.filter((a) => a.has_token_guide);
  } else if (filterType.value === "unconfigured") {
    list = list.filter((a) => !a.has_token_guide);
  }

  const kw = searchKeyword.value.trim().toLowerCase();
  if (!kw) return list;
  return list.filter(
    (a) =>
      a.path.toLowerCase().includes(kw) ||
      a.tool_or_project.toLowerCase().includes(kw) ||
      a.filename.toLowerCase().includes(kw) ||
      a.preview.toLowerCase().includes(kw)
  );
});

// 加载本地文件索引缓存（秒级响应，下次无需重复全盘搜索）
const loadCachedIndexOrScan = async () => {
  try {
    const cache = await invoke<AgentIndexCache>("get_cached_agent_index");
    if (cache && cache.agents && cache.agents.length > 0) {
      agents.value = cache.agents;
      indexUpdatedAt.value = cache.updated_at;
      return;
    }
  } catch (err) {
    console.error("读取本地 agent 索引失败:", err);
  }

  // 若本地尚无索引，则触发首次全盘扫描
  await handleRefreshScan();
};

// 全盘重新扫描并更新本地索引缓存
const handleRefreshScan = async () => {
  isScanning.value = true;
  try {
    const cache = await invoke<AgentIndexCache>("scan_and_refresh_agents");
    agents.value = cache.agents;
    indexUpdatedAt.value = cache.updated_at;
  } catch (err) {
    alert("扫描失败: " + err);
  } finally {
    isScanning.value = false;
  }
};

onMounted(() => {
  loadCachedIndexOrScan();
});

// 一键批量写入所有已扫描文件
const handleInjectAll = async () => {
  if (agents.value.length === 0) return;

  const count = agents.value.length;
  if (
    !confirm(
      `确定要将 GitHub 访问凭据使用说明写入这 ${count} 个 AGENT.md 文件中吗？系统将自动使用防重标记块，安全无害。`
    )
  ) {
    return;
  }

  isInjecting.value = true;
  try {
    const paths = agents.value.map((a) => a.path);
    const summary = await invoke<InjectSummary>("inject_agents", { paths });
    alert(
      `批量写入完成！\n成功: ${summary.success} 个\n失败: ${summary.failed} 个\n所有文件已规范包含 CLI 令牌访问指引！`
    );
    // 重新刷新列表状态
    await handleRefreshScan();
  } catch (err) {
    alert("批量写入失败: " + err);
  } finally {
    isInjecting.value = false;
  }
};

// 单个文件注入
const handleInjectSingle = async (item: AgentFileInfo) => {
  try {
    const summary = await invoke<InjectSummary>("inject_agents", {
      paths: [item.path],
    });
    if (summary.success > 0) {
      item.has_token_guide = true;
      alert(`已成功将 GitHub 凭据指引写入:\n${item.path}`);
    } else {
      alert("写入失败: " + (summary.details[0]?.message || "未知错误"));
    }
  } catch (err) {
    alert("写入失败: " + err);
  }
};

// 在资源管理器中定位
const handleOpenFolder = async (path: string) => {
  try {
    await invoke("open_in_explorer", { path });
  } catch (err) {
    alert("打开文件夹失败: " + err);
  }
};

// 查看/编辑
const handleViewEdit = async (item: AgentFileInfo) => {
  activeEditItem.value = item;
  try {
    const content = await invoke<string>("read_file_content", { path: item.path });
    editFileContent.value = content;
  } catch (err) {
    alert("读取文件内容失败: " + err);
  }
};

// 在弹窗中追加指引
const handleInjectInModal = () => {
  const guideBlock = `\n<!-- AI-HELPER:GITHUB-CREDENTIALS-START -->\n## 🔑 GitHub Access Credentials (AI Helper Automated Guide)\nWhen you need to access private GitHub repositories or perform Git/API operations, you can dynamically retrieve the decrypted GitHub Token via:\n- **PowerShell**:\n  \`\`\`powershell\n  $env:GITHUB_TOKEN = (ai-helper token get)\n  \`\`\`\n- **Bash / Linux / Mac**:\n  \`\`\`bash\n  export GITHUB_TOKEN=$(ai-helper token get)\n  \`\`\`\n- **Direct Output**:\n  \`ai-helper token get\`\n<!-- AI-HELPER:GITHUB-CREDENTIALS-END -->\n`;
  if (!editFileContent.value.includes("<!-- AI-HELPER:GITHUB-CREDENTIALS-START -->")) {
    editFileContent.value = editFileContent.value.trimEnd() + "\n" + guideBlock;
  } else {
    alert("该文件中已存在凭据指引标记块");
  }
};

// 保存编辑的文件
const handleSaveFileContent = async () => {
  if (!activeEditItem.value) return;
  try {
    await invoke("save_file_content", {
      path: activeEditItem.value.path,
      content: editFileContent.value,
    });
    alert("文件保存成功！");
    activeEditItem.value = null;
    await handleRefreshScan();
  } catch (err) {
    alert("保存失败: " + err);
  }
};

// 格式化字节数
const formatBytes = (bytes: number) => {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + " " + sizes[i];
};
</script>

<style scoped>
.agent-manager-root {
  display: flex;
  flex-direction: column;
  height: 100%;
  padding: 16px;
  gap: 14px;
  overflow: hidden;
}

/* 顶部扫描卡片 */
.scan-header-card {
  padding: 12px 16px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-shrink: 0;
  background: rgba(14, 18, 30, 0.95);
  border: 1px solid rgba(0, 240, 255, 0.25);
}

.header-left {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.title-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.neon-icon {
  color: var(--cyber-neon-cyan);
}

.header-title {
  font-size: 13px;
  font-weight: 700;
  color: #fff;
  letter-spacing: 0.5px;
}

.stats-row {
  display: flex;
  gap: 12px;
}

.stat-pill {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 11px;
}

.stat-pill .label {
  color: #94a3b8;
}

.stat-pill .val {
  color: #fff;
  font-weight: 700;
}

.stat-pill .val-green {
  color: var(--cyber-neon-green);
  font-weight: 700;
}

.stat-pill .val-yellow {
  color: var(--cyber-neon-yellow);
  font-weight: 700;
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 10px;
}

.inject-all-btn {
  height: 34px;
  padding: 0 16px;
  font-size: 13px;
}

/* 列表顶部工具栏 */
.list-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-shrink: 0;
}

.toolbar-left {
  display: flex;
  align-items: center;
  gap: 10px;
}

.list-title {
  font-size: 12px;
  font-weight: 700;
  color: #94a3b8;
}

.toolbar-right {
  display: flex;
  align-items: center;
  gap: 12px;
}

.filter-tabs {
  display: flex;
  background: rgba(10, 13, 22, 0.8);
  border: 1px solid rgba(0, 240, 255, 0.15);
  border-radius: 4px;
  padding: 2px;
}

.tab-btn {
  background: transparent;
  border: none;
  color: #64748b;
  font-size: 11px;
  padding: 4px 10px;
  border-radius: 3px;
  cursor: pointer;
  transition: all 0.15s ease;
}

.tab-btn.active {
  background: rgba(0, 240, 255, 0.18);
  color: var(--cyber-neon-cyan);
  font-weight: 600;
}

.search-input-box {
  position: relative;
  width: 240px;
}

.search-icon {
  position: absolute;
  left: 10px;
  top: 50%;
  transform: translateY(-50%);
  color: #64748b;
}

.search-input {
  height: 30px;
  padding-left: 30px;
  font-size: 12px;
}

/* 列表滚动与长条卡片 */
.agent-scroll-list {
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

.agent-row-card {
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

.agent-row-card:hover {
  background: rgba(18, 23, 38, 0.95);
  border-color: rgba(0, 240, 255, 0.4);
}

.row-left-bar {
  position: absolute;
  left: 0;
  top: 15%;
  bottom: 15%;
  width: 3px;
  border-radius: 2px;
}

.bar-green {
  background: var(--cyber-neon-green);
  box-shadow: 0 0 6px var(--cyber-neon-green);
}

.bar-yellow {
  background: var(--cyber-neon-yellow);
  box-shadow: 0 0 6px var(--cyber-neon-yellow);
}

.agent-info-main {
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

.tool-badge {
  font-weight: 700;
}

.file-name {
  font-size: 13px;
  font-weight: 700;
  color: #fff;
}

.file-size {
  font-size: 10px;
  color: #64748b;
}

.update-time {
  font-size: 10px;
  color: #64748b;
  margin-left: auto;
}

.file-path-line {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 11px;
  color: #94a3b8;
  font-family: 'Consolas', monospace;
  overflow: hidden;
  white-space: nowrap;
  text-overflow: ellipsis;
}

.path-icon {
  color: var(--cyber-neon-cyan);
  flex-shrink: 0;
}

.path-text {
  overflow: hidden;
  white-space: nowrap;
  text-overflow: ellipsis;
}

.file-preview-line {
  font-size: 11px;
  color: #64748b;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  font-family: 'Consolas', monospace;
}

.agent-actions {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
}

.spin-anim {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

/* 弹窗样式 */
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(4, 6, 12, 0.8);
  backdrop-filter: blur(6px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 999;
}

.cyber-modal {
  width: 720px;
  height: 520px;
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

.modal-title {
  font-size: 13px;
  font-weight: 700;
  color: #fff;
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

.modal-path-bar {
  padding: 6px 16px;
  background: rgba(8, 11, 20, 0.9);
  border-bottom: 1px solid rgba(0, 240, 255, 0.1);
  font-size: 11px;
  color: var(--cyber-neon-cyan);
  word-break: break-all;
}

.modal-editor-body {
  flex: 1;
  padding: 12px 16px;
  min-height: 0;
}

.modal-textarea {
  width: 100%;
  height: 100%;
  resize: none;
  font-family: 'Consolas', monospace;
  font-size: 12px;
  line-height: 1.5;
}

.modal-footer {
  height: 50px;
  padding: 0 16px;
  border-top: 1px solid rgba(0, 240, 255, 0.12);
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.footer-right {
  display: flex;
  gap: 8px;
}
</style>
