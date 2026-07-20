<script setup lang="ts">
import { onUnmounted, watch } from "vue";
import { animateConfirm, cancelMotion } from "../utils/motion";

const props = withDefaults(
  defineProps<{
    open: boolean;
    title: string;
    message: string;
    confirmLabel?: string;
    cancelLabel?: string;
    danger?: boolean;
  }>(),
  {
    confirmLabel: "确认",
    cancelLabel: "取消",
    danger: false,
  },
);

const emit = defineEmits<{
  confirm: [];
  cancel: [];
}>();

function onMaskClick() {
  emit("cancel");
}

function onConfirm() {
  emit("confirm");
}

function onCancel() {
  emit("cancel");
}

function onKeydown(e: KeyboardEvent) {
  if (e.key === "Escape") {
    e.preventDefault();
    emit("cancel");
  }
}

watch(
  () => props.open,
  (open) => {
    if (open) {
      window.addEventListener("keydown", onKeydown);
    } else {
      window.removeEventListener("keydown", onKeydown);
    }
  },
);

onUnmounted(() => {
  window.removeEventListener("keydown", onKeydown);
});

function onEnter(el: Element, done: () => void) {
  void animateConfirm(el, "in").then(done);
}

function onLeave(el: Element, done: () => void) {
  void animateConfirm(el, "out").then(done);
}

function onEnterCancelled(el: Element) {
  cancelMotion(el);
  const card = el.querySelector(".confirm-card");
  if (card) cancelMotion(card);
}

function onLeaveCancelled(el: Element) {
  cancelMotion(el);
  const card = el.querySelector(".confirm-card");
  if (card) cancelMotion(card);
}
</script>

<template>
  <Teleport to="body">
    <Transition
      :css="false"
      @enter="onEnter"
      @leave="onLeave"
      @enter-cancelled="onEnterCancelled"
      @leave-cancelled="onLeaveCancelled"
    >
      <div
        v-if="open"
        class="confirm-mask"
        role="presentation"
        @click.self="onMaskClick"
      >
        <div
          class="confirm-card"
          role="dialog"
          aria-modal="true"
          aria-labelledby="confirm-title"
          aria-describedby="confirm-message"
          @click.stop
        >
          <h2 id="confirm-title" class="confirm-title">{{ title }}</h2>
          <p id="confirm-message" class="confirm-message">{{ message }}</p>
          <div class="confirm-actions">
            <button type="button" class="btn-secondary" @click="onCancel">
              {{ cancelLabel }}
            </button>
            <button
              type="button"
              :class="danger ? 'btn-danger' : 'btn-primary'"
              @click="onConfirm"
            >
              {{ confirmLabel }}
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.confirm-mask {
  position: fixed;
  inset: 0;
  z-index: 50;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 24px 18px;
  background: color-mix(in srgb, var(--fg) 42%, transparent);
  backdrop-filter: blur(2px);
}

.confirm-card {
  width: min(100%, 340px);
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius-card);
  padding: 18px 16px 14px;
  box-shadow: 0 12px 40px rgba(44, 36, 22, 0.16);
  will-change: transform, opacity;
}

.confirm-title {
  margin: 0 0 8px;
  font-family: var(--font-display);
  font-size: var(--fs-h3);
  font-weight: 650;
  letter-spacing: -0.015em;
  line-height: 1.3;
  color: var(--fg);
}

.confirm-message {
  margin: 0 0 16px;
  font-size: 14px;
  line-height: 1.45;
  color: var(--muted);
  white-space: pre-wrap;
}

.confirm-actions {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 8px;
}

.confirm-actions .btn-primary,
.confirm-actions .btn-secondary,
.confirm-actions .btn-danger {
  width: 100%;
  min-height: 44px;
  padding: 10px 12px;
  font-size: 14px;
}
</style>
