import { useMemo } from "react";
import { useRadarStore } from "../store/radarStore";

export function PlayerSidePanel() {
  const players = useRadarStore((state) => state.latest.players);
  const localTeam = useRadarStore((state) => state.latest.match_state.local_team);

  const [friendlies, enemies] = useMemo(() => {
    const f = players.filter((p) => p.team === localTeam);
    const e = players
      .filter((p) => p.team !== localTeam)
      .sort((a, b) => b.threat_score - a.threat_score);
    return [f, e];
  }, [players, localTeam]);

  return (
    <aside className="side-panel">
      <section>
        <h3>Friendlies</h3>
        <ul>
          {friendlies.map((player) => (
            <li key={player.entity_id}>
              <span>{player.steam_name}</span>
              <span>{player.health} HP</span>
            </li>
          ))}
        </ul>
      </section>

      <section>
        <h3>Threat board</h3>
        <ul>
          {enemies.map((player) => (
            <li key={player.entity_id}>
              <span>{player.steam_name}</span>
              <span className={`band-${player.threat_band}`}>{player.threat_band}</span>
            </li>
          ))}
        </ul>
      </section>
    </aside>
  );
}
