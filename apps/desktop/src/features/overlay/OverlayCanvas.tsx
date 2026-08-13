import { useEffect, useState } from "react";
import type { OverlayUpdate } from "../../lib/contracts";
import { desktop } from "../../lib/tauri";

export function OverlayCanvas() {
  const [update, setUpdate] = useState<OverlayUpdate | null>(null);

  useEffect(() => {
    let latest = 0;
    let timer: number | undefined;
    let dispose: (() => void) | undefined;
    void desktop
      .onOverlay((next) => {
        if (next.generation < latest) return;
        latest = next.generation;
        setUpdate(next);
        window.clearTimeout(timer);
        timer = window.setTimeout(() => {
          setUpdate(null);
        }, next.expires_after_ms);
      })
      .then((unlisten) => {
        dispose = unlisten;
      });
    return () => {
      window.clearTimeout(timer);
      dispose?.();
    };
  }, []);

  if (!update) return <div className="overlay-root" aria-hidden="true" />;
  return (
    <svg className="overlay-root" viewBox="0 0 1000 1000" aria-hidden="true">
      <defs>
        <marker
          id="arrow"
          markerWidth="9"
          markerHeight="9"
          refX="8"
          refY="4.5"
          orient="auto"
        >
          <path d="M0,0 L9,4.5 L0,9 Z" className="overlay-fill" />
        </marker>
      </defs>
      {update.elements.map((element, index) => {
        if (element.kind === "rect") {
          return (
            <g key={index}>
              <rect
                className="overlay-shape"
                x={element.bounds.x * 1000}
                y={element.bounds.y * 1000}
                width={element.bounds.width * 1000}
                height={element.bounds.height * 1000}
                rx="18"
              />
              {element.label && (
                <text
                  className="overlay-label"
                  x={element.bounds.x * 1000}
                  y={Math.max(35, element.bounds.y * 1000 - 16)}
                >
                  {element.label}
                </text>
              )}
            </g>
          );
        }
        if (element.kind === "arrow") {
          return (
            <g key={index}>
              <line
                className="overlay-arrow"
                x1={element.from.x * 1000}
                y1={element.from.y * 1000}
                x2={element.to.x * 1000}
                y2={element.to.y * 1000}
                markerEnd="url(#arrow)"
              />
              {element.label && (
                <text
                  className="overlay-label"
                  x={element.from.x * 1000}
                  y={element.from.y * 1000 - 16}
                >
                  {element.label}
                </text>
              )}
            </g>
          );
        }
        const point = element.at;
        return (
          <g key={index}>
            <circle
              className="overlay-point"
              cx={point.x * 1000}
              cy={point.y * 1000}
              r={element.kind === "step" ? 30 : 18}
            />
            {element.kind === "step" && (
              <text
                className="overlay-number"
                x={point.x * 1000}
                y={point.y * 1000 + 9}
              >
                {element.number}
              </text>
            )}
            <text
              className="overlay-label"
              x={Math.min(820, point.x * 1000 + 42)}
              y={Math.max(35, point.y * 1000 + 8)}
            >
              {element.label}
            </text>
          </g>
        );
      })}
    </svg>
  );
}
