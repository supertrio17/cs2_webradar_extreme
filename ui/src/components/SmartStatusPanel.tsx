import { useRadarStore } from "../store/radarStore";

export function SmartStatusPanel() {
  const snapshot = useRadarStore((state) => state.latest);

  const utilityByTeam = snapshot.grenades.reduce(
    (acc, grenade) => {
      if (grenade.owner_team === "counter_terrorists") acc.ct += 1;
      if (grenade.owner_team === "terrorists") acc.t += 1;
      return acc;
    },
    { t: 0, ct: 0 }
  );

  return (
    <section className="smart-panel">
      <div>
        <h4>Bomb predictor</h4>
        {snapshot.bomb?.status === "planted" ? (
          <p>
            {snapshot.bomb.site} site · {snapshot.bomb.survival_band ?? "unknown"} ·
            {" "}
            {Math.round(snapshot.bomb.predicted_damage ?? 0)} dmg
          </p>
        ) : (
          <p>No planted bomb</p>
        )}
      </div>
      <div>
        <h4>Live utility</h4>
        <p>
          CT {utilityByTeam.ct} · T {utilityByTeam.t}
        </p>
      </div>
      <div>
        <h4>Staleness</h4>
        <p>{snapshot.diagnostics.stale_entities} entities stale</p>
      </div>
    </section>
  );
}
