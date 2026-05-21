import { ref, computed } from "vue";
import { ru } from "@/locales/ru";
import { en } from "@/locales/en";
import type { Translations } from "@/locales/ru";

export type Lang = "ru" | "en";

const locales: Record<Lang, Translations> = { ru, en };
const locale = ref<Lang>("en");

function setLocale(lang: Lang): void {
  locale.value = lang;
}

export function useI18n() {
  const i18n = computed(() => locales[locale.value]);
  return { locale, i18n, setLocale };
}
