type TauriInvoke = (cmd: string, payload?: Record<string, unknown>) => Promise<unknown>;

function getInvoke(): TauriInvoke | null {
  const tauri = (window as Window & { __TAURI__?: { core?: { invoke?: TauriInvoke } } }).__TAURI__;
  return tauri?.core?.invoke ?? null;
}

export async function refreshOffsetsAndSchemas(): Promise<string> {
  const invoke = getInvoke();
  if (!invoke) {
    throw new Error("Desktop bridge unavailable");
  }

  const result = await invoke("dump_refresh", {});
  return String(result ?? "Refresh requested");
}
