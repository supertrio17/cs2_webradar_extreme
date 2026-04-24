import { useRadarStore } from "../store/radarStore";

export function TopTelemetryStrip() {
  const snapshot = useRadarStore((state) => state.latest);

  return (
    <header className="telemetry-strip">
      <div>
        <strong>{snapshot.match_state.map_name}</strong>
        <span>Round {snapshot.match_state.round}</span>
      </div>
      <div>
        <span>T {snapshot.match_state.score_t}</span>
        <span>:</span>
        <span>CT {snapshot.match_state.score_ct}</span>
      </div>
      <div>
        <span>Status: {snapshot.diagnostics.engine_status}</span>
        <span>Build: {snapshot.diagnostics.build_number ?? "unknown"}</span>
      </div>
    </header>
  );
}
