import { useState } from "react";
import { refreshOffsetsAndSchemas } from "../lib/desktopCommands";
import { useSettingsStore } from "../store/settingsStore";

export function SettingsDrawer() {
  const [open, setOpen] = useState(false);
  const [refreshMessage, setRefreshMessage] = useState<string>("");
  const showThreatRings = useSettingsStore((state) => state.showThreatRings);
  const showGrenadeTimers = useSettingsStore((state) => state.showGrenadeTimers);
  const overlayMode = useSettingsStore((state) => state.overlayMode);
  const set = useSettingsStore((state) => state.set);

  const handleRefresh = async () => {
    try {
      const result = await refreshOffsetsAndSchemas();
      setRefreshMessage(result);
    } catch (error) {
      setRefreshMessage((error as Error).message);
    }
  };

  return (
    <div className={`settings-drawer ${open ? "open" : ""}`}>
      <button className="settings-toggle" onClick={() => setOpen((v) => !v)}>
        {open ? "Close" : "Settings"}
      </button>
      {open && (
        <div className="settings-content">
          <label>
            <input
              type="checkbox"
              checked={showThreatRings}
              onChange={(event) => set("showThreatRings", event.target.checked)}
            />
            Threat rings
          </label>
          <label>
            <input
              type="checkbox"
              checked={showGrenadeTimers}
              onChange={(event) => set("showGrenadeTimers", event.target.checked)}
            />
            Grenade timers
          </label>
          <label>
            <input
              type="checkbox"
              checked={overlayMode}
              onChange={(event) => set("overlayMode", event.target.checked)}
            />
            Overlay profile
          </label>
          <button className="refresh-button" onClick={handleRefresh}>
            Refresh Offsets/Schemas
          </button>
          {refreshMessage ? <small>{refreshMessage}</small> : null}
        </div>
      )}
    </div>
  );
}
