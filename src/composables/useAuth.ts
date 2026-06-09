import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { ref } from "vue";

// Запрос credentials от askpass-моста (см. src-tauri/src/askpass.rs).
export interface AskpassRequest {
  id: number;
  prompt: string;
  kind: "username" | "password" | "passphrase" | "confirm" | "generic";
  host?: string | null;
  key_path?: string | null;
}

const current = ref<AskpassRequest | null>(null);
const queue: AskpassRequest[] = [];

function advance() {
  current.value = queue.shift() ?? null;
}

listen<AskpassRequest>("askpass_request", (event) => {
  if (current.value) queue.push(event.payload);
  else current.value = event.payload;
});

async function respond(id: number, value: string, cancel: boolean) {
  await invoke("askpass_respond", { id, value, cancel });
}

export async function submitAskpass(value: string, remember: boolean) {
  const req = current.value;
  if (!req) return;
  // «Запомнить» относится только к HTTPS-логину/паролю — включаем git credential
  // helper, чтобы git сам сохранил введённое после успешной операции.
  if (remember && (req.kind === "password" || req.kind === "username")) {
    invoke("ensure_credential_helper").catch(() => {});
  }
  await respond(req.id, value, false);
  advance();
}

export async function cancelAskpass() {
  const req = current.value;
  if (!req) return;
  await respond(req.id, "", true);
  advance();
}

export function useAuth() {
  return { current, submitAskpass, cancelAskpass };
}
