<script setup lang="ts">
import { ref, computed, nextTick, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";
import { Microphone, Download, Delete, VideoPlay, VideoPause, Clock, ArrowDown } from "@element-plus/icons-vue";

// ─────────────────────────────────────────────
// 类型定义
// ─────────────────────────────────────────────
interface HistoryItem {
  id: string;
  text: string;
  /** 后台返回的 WAV 文件路径或 blob URL */
  audioSrc: string;
  createdAt: Date;
}

// ─────────────────────────────────────────────
// 状态变量
// ─────────────────────────────────────────────

/** 待合成的文本 */
const inputText = ref<string>("");

/** 是否正在生成中 */
const isGenerating = ref<boolean>(false);

/** 状态说明文字 */
const statusMessage = ref<string>("");

/** 状态类型：success | error | "" */
const statusType = ref<"success" | "danger" | "">("");

/** 当前播放器的音频路径/blob URL */
const audioSrc = ref<string>("");

/** 模板 ref，指向 <audio> 元素 */
const audioEl = ref<HTMLAudioElement | null>(null);

/** 音频是否正在播放 */
const isPlaying = ref<boolean>(false);

/** 当前正在播放的历史条目 id */
const currentPlayingId = ref<string | null>(null);

/** 历史记录（从 localStorage 恢复） */
const history = ref<HistoryItem[]>(
  (() => {
    try {
      const raw = localStorage.getItem("tts_history");
      if (!raw) return [];
      return (JSON.parse(raw) as any[]).map((item) => ({
        ...item,
        createdAt: new Date(item.createdAt),
      })) as HistoryItem[];
    } catch {
      return [];
    }
  })()
);

/** 监听历史记录变化，自动持久化到 localStorage（最多保留 30 条） */
watch(
  history,
  (val) => {
    try {
      localStorage.setItem("tts_history", JSON.stringify(val.slice(0, 30)));
    } catch {
      // localStorage 空间不足时静默忽略
    }
  },
  { deep: true }
);

/** 音频格式 */
const audioFormat = ref<"mp3" | "wav">(localStorage.getItem("tts_audio_format") === "wav" ? "wav" : "mp3");

/** 是否启用 API Key */
const useApiKey = ref<boolean>(localStorage.getItem("tts_use_api_key") === "true");

/** 用户输入的 API Key */
const apiKey = ref<string>(localStorage.getItem("tts_api_key") ?? "");

/** 语速，范围 -50 ~ 50，默认 0 */
const speed = ref<number>(Number(localStorage.getItem("tts_speed") ?? "0"));

watch(useApiKey, (val) => localStorage.setItem("tts_use_api_key", String(val)));
watch(apiKey, (val) => localStorage.setItem("tts_api_key", val));
watch(speed, (val) => localStorage.setItem("tts_speed", String(val)));

/** 设置面板是否展开，默认展开 */
const settingsExpanded = ref<boolean>(localStorage.getItem("tts_settings_expanded") !== "false");
watch(settingsExpanded, (val) => localStorage.setItem("tts_settings_expanded", String(val)));

// ─────────────────────────────────────────────
// 计算属性
// ─────────────────────────────────────────────
const canGenerate = computed(
  () => inputText.value.trim().length > 0 && !isGenerating.value && (!useApiKey.value || apiKey.value.trim().length > 0)
);
const textLength = computed(() => inputText.value.length);

// ─────────────────────────────────────────────
// 占位函数 —— 请在此处实现后台调用逻辑
// ─────────────────────────────────────────────

/**
 * 触发语音合成，调用后台 API，拿到 WAV 文件路径后更新播放器。
 * 实现时应：
 *  1. 调用 invoke("synthesize_speech", { text: inputText.value })
 *  2. 将返回的 WAV 路径赋值给 audioSrc
 *  3. 调用 addToHistory(wavPath) 写入历史
 */
async function generateSpeech(): Promise<void> {
  if (!canGenerate.value) return;
  isGenerating.value = true;
  statusMessage.value = "";
  statusType.value = "";

  try {
    const dataUrl = await invoke<string>("synthesize_speech", {
      text: inputText.value,
      format: audioFormat.value,
      apiKey: useApiKey.value ? apiKey.value : "",
      speed: speed.value,
    });
    audioSrc.value = dataUrl;
    addToHistory(dataUrl);
    // 保存输出格式及 api key 设置
    localStorage.setItem("tts_audio_format", audioFormat.value);
    localStorage.setItem("tts_use_api_key", String(useApiKey.value));
    if (useApiKey.value) localStorage.setItem("tts_api_key", apiKey.value);
    statusMessage.value = "合成完成";
    statusType.value = "success";
  } catch (e) {
    statusMessage.value = `合成失败：${e}`;
    statusType.value = "danger";
  } finally {
    isGenerating.value = false;
  }
}

/**
 * 将本次合成写入历史记录。
 * @param src WAV 文件路径或 blob URL
 */
function addToHistory(src: string): void {
  history.value.unshift({
    id: Date.now().toString(),
    text: inputText.value,
    audioSrc: src,
    createdAt: new Date(),
  });
}

/**
 * 播放 / 暂停历史条目。
 * @param item 历史记录条目
 */
function togglePlayHistoryItem(item: HistoryItem): void {
  if (currentPlayingId.value === item.id && isPlaying.value) {
    // 暂停
    audioEl.value?.pause();
    isPlaying.value = false;
    currentPlayingId.value = null;
  } else {
    currentPlayingId.value = item.id;
    if (audioSrc.value === item.audioSrc) {
      // 同一首，直接播放
      audioEl.value?.play();
    } else {
      // 切换音源，等 DOM 更新后再播
      audioSrc.value = item.audioSrc;
      nextTick(() => {
        audioEl.value?.load();
        audioEl.value?.play();
      });
    }
    isPlaying.value = true;
  }
}

/**
 * 下载 WAV 文件。
 * @param item 若不传则下载当前 audioSrc
 */
async function downloadAudio(item?: HistoryItem): Promise<void> {
  const src = item?.audioSrc ?? audioSrc.value;
  if (!src) return;
  try {
    await invoke("save_audio", { url: src });
  } catch (e) {
    statusMessage.value = `下载失败：${e}`;
    statusType.value = "danger";
  }
}

/** 删除历史条目 */
function deleteHistoryItem(id: string): void {
  history.value = history.value.filter((i) => i.id !== id);
  if (currentPlayingId.value === id) {
    isPlaying.value = false;
    currentPlayingId.value = null;
  }
}

/** 清空全部历史 */
function clearHistory(): void {
  history.value = [];
  isPlaying.value = false;
  currentPlayingId.value = null;
}

/** 格式化时间 */
function formatTime(date: Date): string {
  return date.toLocaleTimeString("zh-CN", { hour: "2-digit", minute: "2-digit", second: "2-digit" });
}
</script>

<template>
  <div class="app-wrapper">
    <!-- ── 顶部标题栏 ── -->
    <header class="top-bar">
      <div class="brand">
        <el-icon class="brand-icon"><Microphone /></el-icon>
        <span class="brand-title">曼波语音生成器</span>
        <el-tag size="small" effect="dark" type="info" class="brand-tag">TTS</el-tag>
        <span class="brand-version">v0.1.2</span>
      </div>
    </header>

    <!-- ── 主体布局 ── -->
    <main class="main-layout">
      <!-- ══ 左列：输入 + 生成 ══ -->
      <section class="panel left-panel">
        <!-- 文本输入区 -->
        <div class="panel-section input-section">
          <div class="section-label">
            <span>输入文本</span>
            <span class="char-count">{{ textLength }} 字</span>
          </div>
          <el-input
            v-model="inputText"
            type="textarea"
            :rows="6"
            placeholder="在此输入要转换为语音的文字内容…"
            resize="none"
            class="text-input"
            :disabled="isGenerating"
          />
        </div>

        <!-- 格式 / 语速 / API Key -->
        <div class="panel-section settings-card">
          <!-- 标题行，点击折叠 -->
          <div class="setting-row settings-header" @click="settingsExpanded = !settingsExpanded">
            <span class="setting-label">设置</span>
            <el-icon class="settings-arrow" :class="{ collapsed: !settingsExpanded }"><ArrowDown /></el-icon>
          </div>

          <transition name="settings-collapse">
            <div v-show="settingsExpanded" class="settings-body">
              <!-- 输出格式 -->
              <div class="setting-divider" />
              <div class="setting-row">
                <span class="setting-label">输出格式</span>
                <el-radio-group v-model="audioFormat" :disabled="isGenerating" class="format-group">
                  <el-radio-button value="mp3">MP3</el-radio-button>
                  <el-radio-button value="wav">WAV</el-radio-button>
                </el-radio-group>
              </div>

              <div class="setting-divider" />

              <!-- 语速 -->
              <div class="setting-row">
                <span class="setting-label">语速</span>
                <span class="speed-value">{{ speed > 0 ? `+${speed}` : speed }}</span>
              </div>
              <el-slider
                v-model="speed"
                :min="-50"
                :max="50"
                :step="1"
                :disabled="isGenerating"
                class="speed-slider"
              />

              <div class="setting-divider" />

              <!-- API Key -->
              <div class="setting-row">
                <span class="setting-label">API Key</span>
                <div style="display:flex;align-items:center;gap:8px;">
                  <el-button size="small" text type="primary" @click="openUrl('https://api.milorapart.top/')">获取 API Key</el-button>
                  <el-checkbox v-model="useApiKey" :disabled="isGenerating">启用 API Key</el-checkbox>
                </div>
              </div>
              <el-input
                v-model="apiKey"
                type="password"
                show-password
                placeholder="输入 Bearer Token…"
                :disabled="!useApiKey || isGenerating"
                class="key-input"
              />
            </div>
          </transition>
        </div>

        <!-- 生成按钮 + 状态 -->
        <div class="panel-section generate-section">
          <el-button
            type="primary"
            size="large"
            class="generate-btn"
            :loading="isGenerating"
            :disabled="!canGenerate"
            @click="generateSpeech"
          >
            <el-icon v-if="!isGenerating"><Microphone /></el-icon>
            {{ isGenerating ? "合成中…" : "生成语音" }}
          </el-button>

          <transition name="fade">
            <div v-if="isGenerating" class="progress-area">
              <div class="progress-indeterminate">
                <div class="progress-bar-anim"></div>
              </div>
              <span class="status-msg">正在调用 API 合成语音…</span>
            </div>
            <div v-else-if="statusMessage" class="status-line">
              <el-text :type="statusType || undefined" size="small">{{ statusMessage }}</el-text>
            </div>
          </transition>
        </div>
      </section>

      <!-- ══ 右列：播放器 + 历史 ══ -->
      <section class="panel right-panel">
        <!-- 音频播放器 -->
        <div class="panel-section">
          <div class="section-label">
            <span>播放器</span>
            <el-button
              v-if="audioSrc"
              size="small"
              :icon="Download"
              type="primary"
              plain
              @click="downloadAudio()"
            >下载</el-button>
          </div>

          <div class="player-card" :class="{ active: !!audioSrc }">
            <div v-if="!audioSrc" class="player-empty">
              <el-icon class="player-empty-icon"><Microphone /></el-icon>
              <span>合成完成后在此播放</span>
            </div>
            <template v-else>
              <audio
                ref="audioEl"
                :src="audioSrc"
                controls
                controlsList="nodownload"
                class="native-audio"
                @play="isPlaying = true"
                @pause="isPlaying = false"
                @ended="isPlaying = false; currentPlayingId = null"
              ></audio>
            </template>
          </div>
        </div>

        <!-- 历史记录 -->
        <div class="panel-section history-section">
          <div class="section-label">
            <span><el-icon style="margin-right:4px;vertical-align:-2px"><Clock /></el-icon>历史记录</span>
            <el-button
              v-if="history.length"
              size="small"
              :icon="Delete"
              text
              type="danger"
              @click="clearHistory"
            >清空</el-button>
          </div>

          <div v-if="!history.length" class="history-empty">
            暂无生成记录
          </div>

          <el-scrollbar v-else style="flex:1;min-height:0;">
            <transition-group name="list" tag="div" class="history-list">
              <div
                v-for="item in history"
                :key="item.id"
                class="history-item"
                :class="{ 'history-item--playing': currentPlayingId === item.id }"
              >
                <div class="history-item-body">
                  <p class="history-text" :title="item.text">{{ item.text }}</p>
                  <div class="history-meta">
                    <span>{{ formatTime(item.createdAt) }}</span>
                  </div>
                </div>
                <div class="history-item-actions">
                  <el-tooltip content="播放 / 暂停" placement="top">
                    <el-button
                      circle
                      size="small"
                      :type="currentPlayingId === item.id && isPlaying ? 'warning' : 'primary'"
                      @click="togglePlayHistoryItem(item)"
                    >
                      <el-icon>
                        <VideoPause v-if="currentPlayingId === item.id && isPlaying" />
                        <VideoPlay v-else />
                      </el-icon>
                    </el-button>
                  </el-tooltip>
                  <el-tooltip content="下载" placement="top">
                    <el-button circle size="small" @click="downloadAudio(item)">
                      <el-icon><Download /></el-icon>
                    </el-button>
                  </el-tooltip>
                  <el-tooltip content="删除" placement="top">
                    <el-button circle size="small" type="danger" @click="deleteHistoryItem(item.id)">
                      <el-icon><Delete /></el-icon>
                    </el-button>
                  </el-tooltip>
                </div>
              </div>
            </transition-group>
          </el-scrollbar>
        </div>
      </section>
    </main>
  </div>
</template>

<style scoped>
/* ─── 全局容器 ─── */
.app-wrapper {
  height: 100vh;
  overflow: hidden;
  background: linear-gradient(135deg, #0d2b52 0%, #174981 60%, #1a5fa8 100%);
  display: flex;
  flex-direction: column;
  color: #e8f0fb;
  font-family: "PingFang SC", "Microsoft YaHei", sans-serif;
}

/* ─── 顶部栏 ─── */
.top-bar {
  height: 44px;
  padding: 0 20px;
  display: flex;
  align-items: center;
  background: rgba(0, 0, 0, 0.25);
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
  backdrop-filter: blur(8px);
  flex-shrink: 0;
}

.brand {
  display: flex;
  align-items: center;
  gap: 10px;
}

.brand-icon {
  font-size: 22px;
  color: #60b4ff;
}

.brand-title {
  font-size: 18px;
  font-weight: 700;
  letter-spacing: 1px;
  background: linear-gradient(90deg, #93d0ff, #c8e8ff);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
}

.brand-tag {
  font-size: 10px;
  letter-spacing: 1px;
}

.brand-version {
  font-size: 11px;
  color: #4a7aa8;
  letter-spacing: 0.5px;
}

/* ─── 主布局 ─── */
.main-layout {
  flex: 1;
  min-height: 0;
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
  padding: 12px 16px 16px;
  max-width: 1200px;
  width: 100%;
  margin: 0 auto;
  box-sizing: border-box;
  overflow: hidden;
}

@media (max-width: 600px) {
  .main-layout {
    grid-template-columns: 1fr;
  }
}

/* ─── 面板 ─── */
.panel {
  display: flex;
  flex-direction: column;
  gap: 8px;
  min-height: 0;
  overflow: hidden;
}

.panel-section {
  background: rgba(255, 255, 255, 0.06);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 12px;
  padding: 10px 14px;
  backdrop-filter: blur(6px);
}

.section-label {
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: 12px;
  font-weight: 600;
  color: #93c5fd;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  margin-bottom: 8px;
}

.char-count {
  font-size: 12px;
  color: #6ea8d8;
  font-weight: 400;
}

/* ─── API Key 输入 ─── */
.key-input :deep(.el-input__wrapper) {
  background: rgba(0, 0, 0, 0.25);
  border-color: rgba(255, 255, 255, 0.15);
  border-radius: 10px;
}

.key-input :deep(.el-input__inner) {
  color: #e0eeff;
  font-family: monospace;
  letter-spacing: 1px;
}

.key-input :deep(.el-input__inner::placeholder) {
  color: #5a7fa8;
  font-family: inherit;
  letter-spacing: 0;
}

.text-input :deep(.el-textarea__inner:disabled),
.text-input :deep(.el-textarea__inner[disabled]) {
  background: rgba(0, 0, 0, 0.25) !important;
  color: #7a9fc0 !important;
  -webkit-text-fill-color: #7a9fc0 !important;
  cursor: not-allowed;
}

.key-input :deep(.el-input__wrapper.is-disabled) {
  background: rgba(0, 0, 0, 0.2) !important;
  border-color: rgba(255, 255, 255, 0.08) !important;
  box-shadow: none !important;
}

.key-input :deep(.el-input__inner:disabled) {
  color: #4a6a88 !important;
  -webkit-text-fill-color: #4a6a88 !important;
}

.key-eye {
  cursor: pointer;
  color: #5a7fa8;
  font-size: 16px;
  transition: color 0.2s;
}

.key-eye:hover {
  color: #93c5fd;
}

.key-hint {
  font-size: 11px;
  color: #5a7fa8;
  margin-top: 6px;
  line-height: 1.5;
}

.key-link {
  color: #60b4ff;
  cursor: pointer;
  text-decoration: underline;
  text-underline-offset: 2px;
}

.key-link:hover {
  color: #93d0ff;
}

/* ─── 设置卡片 ─── */
.settings-card {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.settings-header {
  cursor: pointer;
  user-select: none;
}

.settings-header:hover .setting-label {
  color: #c0d8f8;
}

.settings-arrow {
  color: #93c5fd;
  font-size: 13px;
  transition: transform 0.25s ease;
}

.settings-arrow.collapsed {
  transform: rotate(-90deg);
}

.settings-body {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.settings-collapse-enter-active,
.settings-collapse-leave-active {
  transition: opacity 0.25s ease, transform 0.25s ease;
  transform-origin: top;
}

.settings-collapse-enter-from,
.settings-collapse-leave-to {
  opacity: 0;
  transform: scaleY(0.9);
}

.setting-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.setting-label {
  font-size: 12px;
  font-weight: 600;
  color: #93c5fd;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.setting-divider {
  height: 1px;
  background: rgba(255, 255, 255, 0.07);
  margin: 2px 0;
}


.speed-value {
  font-size: 12px;
  color: #60b4ff;
  font-weight: 600;
  font-variant-numeric: tabular-nums;
}

.speed-slider :deep(.el-slider__runway) {
  background: rgba(255, 255, 255, 0.12);
}

.speed-slider :deep(.el-slider__bar) {
  background: #2563eb;
}

.speed-slider :deep(.el-slider__button) {
  border-color: #60b4ff;
  background: #1a3a5c;
}

/* el-checkbox 暗色适配 */
:deep(.el-checkbox__label) {
  color: #93c5fd;
  font-size: 12px;
}

:deep(.el-checkbox__inner) {
  background: rgba(0, 0, 0, 0.25);
  border-color: rgba(255, 255, 255, 0.25);
}

:deep(.el-checkbox.is-checked .el-checkbox__inner) {
  background: #2563eb;
  border-color: #2563eb;
}

.format-group :deep(.el-radio-button__inner) {
  background: rgba(0, 0, 0, 0.25);
  border-color: rgba(255, 255, 255, 0.15);
  color: #7ab5e0;
}

.format-group :deep(.el-radio-button__original-radio:checked + .el-radio-button__inner) {
  background: #2563eb;
  border-color: #2563eb;
  color: #fff;
  box-shadow: -1px 0 0 0 #2563eb;
}

.format-group :deep(.el-radio-button.is-disabled .el-radio-button__inner) {
  background: rgba(0, 0, 0, 0.2) !important;
  border-color: rgba(255, 255, 255, 0.08) !important;
  color: #4a6a88 !important;
}

.format-group :deep(.el-radio-button.is-disabled.is-active .el-radio-button__inner) {
  background: #1a4aaa !important;
  border-color: #1a4aaa !important;
  color: #8ab4d8 !important;
}

.speed-slider :deep(.el-slider.is-disabled .el-slider__runway) {
  background: rgba(255, 255, 255, 0.08);
}

.speed-slider :deep(.el-slider.is-disabled .el-slider__bar) {
  background: #1a4aaa;
}

.speed-slider :deep(.el-slider.is-disabled .el-slider__button) {
  border-color: #3a6a9a;
  background: #1a3a5c;
}

/* ─── 文本输入 ─── */
.input-section {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
}

.input-section .section-label {
  flex-shrink: 0;
}

.text-input {
  flex: 1;
  min-height: 0;
}

.text-input :deep(.el-textarea) {
  height: 100%;
}

.text-input :deep(.el-textarea__inner) {
  height: 100% !important;
  background: rgba(0, 0, 0, 0.25);
  border: 1px solid rgba(255, 255, 255, 0.15);
  color: #e0eeff;
  border-radius: 10px;
  font-size: 13px;
  line-height: 1.6;
  resize: none;
}

.text-input :deep(.el-textarea__inner:focus) {
  border-color: #60b4ff;
  box-shadow: 0 0 0 2px rgba(96, 180, 255, 0.2);
}

.text-input :deep(.el-textarea__inner::placeholder) {
  color: #5a7fa8;
}

/* ─── 生成按钮区 ─── */
.generate-section {
  display: flex;
  flex-direction: column;
  align-items: stretch;
  gap: 12px;
}

.generate-btn {
  width: 100%;
  height: 40px;
  font-size: 14px;
  font-weight: 600;
  letter-spacing: 1px;
  border-radius: 10px;
  background: linear-gradient(135deg, #2563eb, #1d9bf0);
  border: none;
  box-shadow: 0 4px 16px rgba(37, 99, 235, 0.4);
  transition: box-shadow 0.2s;
}

.generate-btn:not(:disabled):hover {
  box-shadow: 0 6px 20px rgba(37, 99, 235, 0.6);
}

.progress-area {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.progress-indeterminate {
  height: 6px;
  background: rgba(255, 255, 255, 0.1);
  border-radius: 3px;
  overflow: hidden;
}

.progress-bar-anim {
  height: 100%;
  width: 40%;
  background: linear-gradient(90deg, transparent, #60b4ff, transparent);
  border-radius: 3px;
  animation: slide 1.4s ease-in-out infinite;
}

@keyframes slide {
  0%   { transform: translateX(-100%); }
  100% { transform: translateX(350%); }
}

.status-msg {
  font-size: 12px;
  color: #7ab5e0;
}

.status-line {
  text-align: center;
}

/* ─── 播放器 ─── */
.player-card {
  border-radius: 10px;
  background: rgba(0, 0, 0, 0.2);
  border: 1px dashed rgba(255, 255, 255, 0.12);
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
  transition: border-color 0.3s;
  padding: 10px 14px;
}

.player-card.active {
  border-style: solid;
  border-color: rgba(96, 180, 255, 0.3);
}

.player-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  color: #4a7aa8;
  font-size: 13px;
}

.player-empty-icon {
  font-size: 36px;
  color: #2a5278;
}

.native-audio {
  width: 100%;
  accent-color: #60b4ff;
  border-radius: 8px;
}

/* ─── 历史记录 ─── */
.history-section {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
}

.history-section .section-label {
  flex-shrink: 0;
}

.history-section .el-scrollbar {
  flex: 1;
  min-height: 0;
}

.history-empty {
  text-align: center;
  color: #3d6485;
  font-size: 13px;
  padding: 20px 0;
}

.history-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding-right: 4px;
}

.history-item {
  display: flex;
  align-items: center;
  gap: 10px;
  background: rgba(0, 0, 0, 0.2);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 10px;
  padding: 10px 12px;
  transition: border-color 0.2s, background 0.2s;
}

.history-item:hover {
  background: rgba(0, 0, 0, 0.3);
  border-color: rgba(96, 180, 255, 0.25);
}

.history-item--playing {
  border-color: rgba(96, 180, 255, 0.5);
  background: rgba(37, 99, 235, 0.12);
}

.history-item-body {
  flex: 1;
  min-width: 0;
}

.history-text {
  font-size: 13px;
  color: #c5dff8;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  margin-bottom: 4px;
}

.history-meta {
  font-size: 11px;
  color: #5a7fa8;
}

.history-item-actions {
  display: flex;
  gap: 4px;
  flex-shrink: 0;
}

/* ─── 过渡动画 ─── */
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.3s;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

.list-enter-active,
.list-leave-active {
  transition: all 0.3s ease;
}

.list-enter-from {
  opacity: 0;
  transform: translateY(-10px);
}

.list-leave-to {
  opacity: 0;
  transform: translateX(20px);
}
</style>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

html, body, #app {
  height: 100%;
}

/* Element Plus 暗色适配 */
.el-popper.is-light {
  background: #1a3a5c;
  border-color: rgba(255,255,255,0.1);
}

.el-tooltip__popper {
  font-size: 12px;
}
</style>