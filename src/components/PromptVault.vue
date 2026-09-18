<template>
  <div class="prompt-vault-root">
    <!-- 顶部大文本框与编辑操作区 -->
    <section class="editor-section cyber-card">
      <div class="editor-header">
        <div class="header-left">
          <Sparkles class="neon-icon" :size="16" />
          <span class="editor-title">
            {{ editingId ? "MODIFY PROMPT // 修改提示词" : "NEW PROMPT // 录入提示词" }}
          </span>
          <span v-if="editingId" class="cyber-badge cyber-badge-yellow">正在编辑模式</span>
        </div>

        <div class="editor-actions">
          <button class="cyber-btn" title="从剪贴板快速粘贴" @click="handlePasteFromClipboard">
            <ClipboardPaste :size="14" />
            快速粘贴
          </button>
          <button class="cyber-btn" title="清空输入框" @click="handleClearEditor">
            <Eraser :size="14" />
            清空
          </button>
          <button
            v-if="editingId"
            class="cyber-btn"
            @click="handleCancelEdit"
          >
            取消修改
          </button>
          <button
            class="cyber-btn cyber-btn-primary"
            :disabled="!editorContent.trim()"
            @click="handleSavePrompt"
          >
            <Save :size="14" />
            {{ editingId ? "保存修改" : "存入宝库" }}
          </button>
        </div>
      </div>

      <!-- 标题输入行 -->
      <div class="title-input-row">
        <input
          v-model="editorTitle"
          type="text"
          class="cyber-input title-input"
          placeholder="提示词名称 / 简述（留空将自动截取前15字作为标题）..."
        />
      </div>

      <!-- 大文本输入区域 -->
      <div class="textarea-wrapper">
        <textarea
          ref="textareaRef"
          v-model="editorContent"
          class="cyber-input prompt-textarea"
          placeholder="在此处输入、编写或粘贴您的 AI 提示词指令..."
        ></textarea>
      </div>
    </section>

    <!-- 中下方选择列表区域 -->
    <section class="list-section">
      <!-- 列表顶部工具栏 -->
      <div class="list-toolbar">
        <div class="toolbar-left">
          <span class="list-title">PROMPTS LIST // 提示词长条列表</span>
          <span class="cyber-badge cyber-badge-cyan">{{ filteredPrompts.length }} 项</span>
        </div>
        <div class="toolbar-right">
          <div class="search-input-box">
            <Search :size="14" class="search-icon" />
            <input
              v-model="searchKeyword"
              type="text"
              class="cyber-input search-input"
              placeholder="搜索提示词名称或内容..."
            />
          </div>
        </div>
      </div>

      <!-- 长条列表（垂直滚动） -->
      <div class="prompt-scroll-list">
        <div
          v-if="filteredPrompts.length === 0"
          class="empty-state cyber-card"
        >
          <FileText :size="32" class="empty-icon" />
          <p>暂无提示词数据。请在上方大文本框输入并存入提示词！</p>
        </div>

        <div
          v-for="item in filteredPrompts"
          :key="item.id"
          class="prompt-row-card cyber-card"
          :class="{ 'editing-active': editingId === item.id }"
        >
          <!-- 左侧发光装饰条 -->
          <div class="row-indicator"></div>

          <!-- 中间内容展示区 -->
          <div class="row-main">
            <div class="row-header">
              <span class="row-title">{{ item.title }}</span>
              <span class="row-time">{{ item.updated_at }}</span>
            </div>
            <div class="row-preview" :title="item.content">
              {{ item.content }}
            </div>
          </div>

          <!-- 右侧操作三联按钮：“复制”、“修改”、“删除” -->
          <div class="row-actions">
            <!-- 复制按钮 -->
            <button
              class="cyber-btn"
              :class="{ 'copied-success': copiedId === item.id }"
              title="复制提示词内容至剪贴板"
              @click="handleCopy(item)"
            >
              <Check v-if="copiedId === item.id" :size="14" />
              <Copy v-else :size="14" />
              <span>{{ copiedId === item.id ? "已复制" : "复制" }}</span>
            </button>

            <!-- 修改按钮 -->
            <button
              class="cyber-btn"
              title="将内容回填至上方大文本框进行修改"
              @click="handleStartEdit(item)"
            >
              <Edit3 :size="14" />
              <span>修改</span>
            </button>

            <!-- 删除按钮 -->
            <button
              class="cyber-btn cyber-btn-danger"
              title="删除此条提示词"
              @click="handleDelete(item)"
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
import { ref, computed, onMounted, nextTick } from "vue";
import { invoke } from "@tauri-apps/api/core";
import {
  Sparkles,
  ClipboardPaste,
  Eraser,
  Save,
  Search,
  Copy,
  Edit3,
  Trash2,
  Check,
  FileText,
} from "lucide-vue-next";
import type { PromptItem } from "../types/prompt";

const prompts = ref<PromptItem[]>([]);
const editorTitle = ref("");
const editorContent = ref("");
const editingId = ref<string | null>(null);
const searchKeyword = ref("");
const copiedId = ref<string | null>(null);
const textareaRef = ref<HTMLTextAreaElement | null>(null);

// 读取提示词列表
const loadPrompts = async () => {
  try {
    const list = await invoke<PromptItem[]>("get_prompts");
    prompts.value = list;
  } catch (err) {
    console.error("加载提示词失败:", err);
  }
};

onMounted(() => {
  loadPrompts();
});

// 过滤提示词
const filteredPrompts = computed(() => {
  const kw = searchKeyword.value.trim().toLowerCase();
  if (!kw) return prompts.value;
  return prompts.value.filter(
    (p) =>
      p.title.toLowerCase().includes(kw) ||
      p.content.toLowerCase().includes(kw)
  );
});

// 从剪贴板快速粘贴
const handlePasteFromClipboard = async () => {
  try {
    const text = await navigator.clipboard.readText();
    if (text) {
      if (editorContent.value) {
        editorContent.value += "\n" + text;
      } else {
        editorContent.value = text;
      }
    }
  } catch (err) {
    alert("无法访问系统剪贴板，请使用 Ctrl+V 粘贴");
  }
};

// 清空编辑器
const handleClearEditor = () => {
  editorTitle.value = "";
  editorContent.value = "";
  editingId.value = null;
};

// 取消编辑态
const handleCancelEdit = () => {
  handleClearEditor();
};

// 保存或更新提示词
const handleSavePrompt = async () => {
  if (!editorContent.value.trim()) return;

  let title = editorTitle.value.trim();
  if (!title) {
    // 自动以内容前15字符命名
    title = editorContent.value.trim().replace(/[\r\n]+/g, " ");
    if (title.length > 15) {
      title = title.substring(0, 15) + "...";
    }
  }

  try {
    await invoke("save_prompt", {
      id: editingId.value,
      title,
      content: editorContent.value,
      tags: [],
    });

    handleClearEditor();
    await loadPrompts();
  } catch (err) {
    alert("保存提示词失败: " + err);
  }
};

// 开始编辑：将内容回填至上方大文本框
const handleStartEdit = (item: PromptItem) => {
  editingId.value = item.id;
  editorTitle.value = item.title;
  editorContent.value = item.content;

  nextTick(() => {
    textareaRef.value?.focus();
  });
};

// 复制提示词内容
const handleCopy = async (item: PromptItem) => {
  try {
    await navigator.clipboard.writeText(item.content);
    copiedId.value = item.id;
    setTimeout(() => {
      if (copiedId.value === item.id) {
        copiedId.value = null;
      }
    }, 2000);
  } catch (err) {
    alert("复制失败: " + err);
  }
};

// 删除提示词
const handleDelete = async (item: PromptItem) => {
  if (confirm(`确定要删除提示词 "${item.title}" 吗？`)) {
    try {
      await invoke("delete_prompt", { id: item.id });
      if (editingId.value === item.id) {
        handleClearEditor();
      }
      await loadPrompts();
    } catch (err) {
      alert("删除失败: " + err);
    }
  }
};
</script>

<style scoped>
.prompt-vault-root {
  display: flex;
  flex-direction: column;
  height: 100%;
  padding: 16px;
  gap: 14px;
  overflow: hidden;
}

/* 顶部大文本框编辑区 */
.editor-section {
  padding: 12px 14px;
  display: flex;
  flex-direction: column;
  gap: 10px;
  flex-shrink: 0;
  background: rgba(14, 18, 30, 0.95);
  border: 1px solid rgba(0, 240, 255, 0.25);
}

.editor-header {
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

.editor-title {
  font-size: 13px;
  font-weight: 700;
  color: #fff;
  letter-spacing: 0.5px;
}

.editor-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.title-input-row {
  display: flex;
}

.title-input {
  height: 32px;
  font-size: 13px;
  font-weight: 600;
  color: var(--cyber-neon-cyan);
}

.textarea-wrapper {
  position: relative;
}

.prompt-textarea {
  height: 140px;
  min-height: 120px;
  resize: vertical;
  line-height: 1.6;
  font-family: 'Consolas', monospace, sans-serif;
  font-size: 13px;
}

/* 中下方长条列表区 */
.list-section {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
  gap: 10px;
}

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
  letter-spacing: 0.5px;
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

/* 长条卡片列表与滚动 */
.prompt-scroll-list {
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

.prompt-row-card {
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

.prompt-row-card:hover {
  background: rgba(18, 23, 38, 0.95);
  border-color: rgba(0, 240, 255, 0.4);
}

.prompt-row-card.editing-active {
  border-color: var(--cyber-neon-yellow);
  box-shadow: 0 0 10px rgba(252, 238, 10, 0.25);
}

.row-indicator {
  position: absolute;
  left: 0;
  top: 15%;
  bottom: 15%;
  width: 3px;
  background: var(--cyber-neon-cyan);
  box-shadow: 0 0 6px var(--cyber-neon-cyan);
  border-radius: 2px;
}

.row-main {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.row-header {
  display: flex;
  align-items: center;
  gap: 10px;
}

.row-title {
  font-size: 13px;
  font-weight: 700;
  color: #fff;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.row-time {
  font-size: 10px;
  color: #64748b;
}

.row-preview {
  font-size: 12px;
  color: #94a3b8;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  font-family: 'Consolas', monospace, sans-serif;
}

/* 操作三联按钮 */
.row-actions {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
}

.copied-success {
  background: rgba(0, 255, 157, 0.25) !important;
  border-color: var(--cyber-neon-green) !important;
  color: #fff !important;
  box-shadow: 0 0 8px rgba(0, 255, 157, 0.5) !important;
}
</style>
