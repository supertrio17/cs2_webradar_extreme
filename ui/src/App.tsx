import { useEffect } from "react";
import { PlayerSidePanel } from "./components/PlayerSidePanel";
import { RadarCanvas } from "./components/RadarCanvas";
import { SettingsDrawer } from "./components/SettingsDrawer";
import { SmartStatusPanel } from "./components/SmartStatusPanel";
import { TopTelemetryStrip } from "./components/TopTelemetryStrip";
import { startSnapshotStream } from "./lib/snapshotStream";
import { useRadarStore } from "./store/radarStore";
import { useSettingsStore } from "./store/settingsStore";

export default function App() {
  const update = useRadarStore((state) => state.update);
  const hydrateSettings = useSettingsStore((state) => state.hydrate);

  useEffect(() => {
    hydrateSettings();
  }, [hydrateSettings]);

  useEffect(() => {
    let cleanup: (() => void) | undefined;
    startSnapshotStream((snapshot) => update(snapshot)).then((stop) => {
      cleanup = stop;
    });

    return () => {
      if (cleanup) cleanup();
    };
  }, [update]);

  return (
    <div className="app-shell">
      <TopTelemetryStrip />
      <div className="content-grid">
        <RadarCanvas />
        <PlayerSidePanel />
      </div>
      <SmartStatusPanel />
      <SettingsDrawer />
    </div>
  );
}
