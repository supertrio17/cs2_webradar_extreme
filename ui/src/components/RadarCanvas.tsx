import { useEffect, useRef } from "react";
import { useRadarStore } from "../store/radarStore";
import { useSettingsStore } from "../store/settingsStore";

const size = 680;

const teamColor = (team: string) => {
  if (team === "counter_terrorists") return "#49d4ff";
  if (team === "terrorists") return "#ff7d43";
  return "#8e9ab4";
};

export function RadarCanvas() {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const latest = useRadarStore((state) => state.latest);
  const previous = useRadarStore((state) => state.previous);
  const showThreatRings = useSettingsStore((state) => state.showThreatRings);
  const showGrenadeTimers = useSettingsStore((state) => state.showGrenadeTimers);

  useEffect(() => {
    let raf = 0;
    const render = () => {
      const canvas = canvasRef.current;
      if (!canvas) {
        raf = requestAnimationFrame(render);
        return;
      }

      const ctx = canvas.getContext("2d");
      if (!ctx) {
        raf = requestAnimationFrame(render);
        return;
      }

      const interval = Math.max(1, latest.timestamp_ms - previous.timestamp_ms);
      const alpha = Math.min(1, (Date.now() - latest.timestamp_ms) / interval + 0.2);

      ctx.clearRect(0, 0, size, size);
      const gradient = ctx.createRadialGradient(size / 2, size / 2, 10, size / 2, size / 2, size / 2);
      gradient.addColorStop(0, "rgba(0, 219, 222, 0.15)");
      gradient.addColorStop(1, "rgba(5, 5, 25, 0.95)");
      ctx.fillStyle = gradient;
      ctx.fillRect(0, 0, size, size);

      ctx.strokeStyle = "rgba(96, 224, 255, 0.25)";
      for (let i = 1; i <= 5; i += 1) {
        const radius = i * (size / 10);
        ctx.beginPath();
        ctx.arc(size / 2, size / 2, radius, 0, Math.PI * 2);
        ctx.stroke();
      }

      const scale = 0.45;
      const project = (x: number, y: number) => ({
        x: size / 2 + x * scale,
        y: size / 2 - y * scale
      });

      latest.players.forEach((player) => {
        const previousPlayer = previous.players.find((it) => it.entity_id === player.entity_id);
        const px = previousPlayer ? previousPlayer.position.x : player.position.x;
        const py = previousPlayer ? previousPlayer.position.y : player.position.y;

        const interpolated = project(px + (player.position.x - px) * alpha, py + (player.position.y - py) * alpha);
        const color = teamColor(player.team);

        if (showThreatRings && player.team !== latest.match_state.local_team) {
          ctx.beginPath();
          ctx.arc(interpolated.x, interpolated.y, 8 + player.threat_score * 2.5, 0, Math.PI * 2);
          ctx.strokeStyle = "rgba(255, 89, 156, 0.25)";
          ctx.stroke();
        }

        ctx.beginPath();
        ctx.arc(interpolated.x, interpolated.y, 6, 0, Math.PI * 2);
        ctx.fillStyle = color;
        ctx.shadowBlur = 12;
        ctx.shadowColor = color;
        ctx.fill();
        ctx.shadowBlur = 0;
      });

      latest.grenades.forEach((grenade) => {
        const p = project(grenade.position.x, grenade.position.y);
        ctx.beginPath();
        ctx.arc(p.x, p.y, 5, 0, Math.PI * 2);
        ctx.fillStyle = grenade.kind === "smoke" ? "#77ffd8" : "#ffd369";
        ctx.fill();

        if (showGrenadeTimers) {
          ctx.fillStyle = "#d6e6ff";
          ctx.font = "11px Inter, sans-serif";
          ctx.fillText(`${Math.ceil(grenade.remaining_ms / 1000)}s`, p.x + 8, p.y + 4);
        }
      });

      raf = requestAnimationFrame(render);
    };

    raf = requestAnimationFrame(render);
    return () => cancelAnimationFrame(raf);
  }, [latest, previous, showThreatRings, showGrenadeTimers]);

  return <canvas ref={canvasRef} width={size} height={size} className="radar-canvas" />;
}
