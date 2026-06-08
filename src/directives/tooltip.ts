import type { Directive } from "vue";

// Кастомный тултип в стиле VSCode: тёмная плашка с рамкой, появляется с
// небольшой задержкой рядом с курсором мыши. Один общий DOM-узел на всё
// приложение.

const SHOW_DELAY = 500;

let tipEl: HTMLDivElement | null = null;
let showTimer: ReturnType<typeof setTimeout> | null = null;
let activeEl: HTMLElement | null = null;

interface Binding {
  text: string;
  onEnter: (e: MouseEvent) => void;
  onLeave: () => void;
}

// Последняя позиция курсора в зоне активного элемента — у тултипа от неё
// отсчитывается смещение.
let cursorX = 0;
let cursorY = 0;
const registry = new WeakMap<HTMLElement, Binding>();

function ensureTip(): HTMLDivElement {
  if (!tipEl) {
    tipEl = document.createElement("div");
    tipEl.className = "vscode-tooltip";
    document.body.appendChild(tipEl);
  }
  return tipEl;
}

function place(text: string) {
  const tip = ensureTip();
  tip.textContent = text;
  tip.classList.add("visible");
  const tw = tip.offsetWidth;
  const th = tip.offsetHeight;
  // Рядом с курсором, со смещением вправо-вниз (как нативные подсказки).
  // Если не помещается — отзеркаливаем относительно курсора / прижимаем к краю.
  let left = cursorX + 14;
  if (left + tw > window.innerWidth - 8) left = cursorX - 14 - tw;
  if (left < 8) left = 8;
  let top = cursorY + 18;
  if (top + th > window.innerHeight - 8) top = cursorY - 12 - th;
  if (top < 8) top = 8;
  tip.style.left = `${left}px`;
  tip.style.top = `${top}px`;
}

function hide() {
  if (showTimer) {
    clearTimeout(showTimer);
    showTimer = null;
  }
  if (activeEl) activeEl.removeEventListener("mousemove", trackCursor);
  activeEl = null;
  if (tipEl) tipEl.classList.remove("visible");
  window.removeEventListener("scroll", hide, true);
  window.removeEventListener("wheel", hide, true);
}

function makeEnter(el: HTMLElement) {
  return (e: MouseEvent) => {
    const b = registry.get(el);
    if (!b || !b.text) return;
    cursorX = e.clientX;
    cursorY = e.clientY;
    if (showTimer) clearTimeout(showTimer);
    activeEl = el;
    // До показа следим за курсором, чтобы плашка появилась там, где он сейчас.
    el.addEventListener("mousemove", trackCursor);
    showTimer = setTimeout(() => {
      if (activeEl !== el) return;
      el.removeEventListener("mousemove", trackCursor);
      place(b.text);
      window.addEventListener("scroll", hide, true);
      window.addEventListener("wheel", hide, true);
    }, SHOW_DELAY);
  };
}

function trackCursor(e: MouseEvent) {
  cursorX = e.clientX;
  cursorY = e.clientY;
}

export const tooltip: Directive<HTMLElement, string | null | undefined> = {
  mounted(el, binding) {
    const onEnter = makeEnter(el);
    const onLeave = () => {
      if (activeEl === el) hide();
    };
    registry.set(el, { text: binding.value ?? "", onEnter, onLeave });
    el.addEventListener("mouseenter", onEnter);
    el.addEventListener("mouseleave", onLeave);
    el.addEventListener("mousedown", onLeave);
  },
  updated(el, binding) {
    const b = registry.get(el);
    if (b) {
      b.text = binding.value ?? "";
      if (activeEl === el && tipEl?.classList.contains("visible")) {
        if (b.text) place(b.text);
        else hide();
      }
    }
  },
  unmounted(el) {
    const b = registry.get(el);
    if (b) {
      el.removeEventListener("mouseenter", b.onEnter);
      el.removeEventListener("mouseleave", b.onLeave);
      el.removeEventListener("mousedown", b.onLeave);
    }
    registry.delete(el);
    if (activeEl === el) hide();
  },
};
