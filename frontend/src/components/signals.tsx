import { useState, type ReactNode } from "react";
import { Tooltip } from "./Tooltip";

/** Semantic state axis. Elements pick exactly one axis: state OR tag color. */
export type SignalState = "ok" | "active" | "info" | "muted" | "error";

/**
 * A colored dot with an optional short word next to it. The full sentence
 * lives in the tooltip (hover or tap) and in the accessible name, so the dot
 * never has to shout.
 */
export function StatusDot({
  state,
  text,
  detail,
  className = "",
}: Readonly<{
  state: SignalState;
  text?: string;
  detail: string;
  className?: string;
}>) {
  return (
    <Tooltip label={detail} className={className}>
      <span
        className={`status-chip state-${state}`}
        aria-label={text ? `${text}. ${detail}` : detail}
      >
        <span className="status-dot" aria-hidden="true" />
        {text && <span className="status-text">{text}</span>}
      </span>
    </Tooltip>
  );
}

/**
 * Value-first metadata chip: shows the value, reveals the key on hover.
 * Example: "340 pages" with tooltip "Page count".
 */
export function MetaChip({
  label,
  children,
  href,
  icon,
  className = "",
}: Readonly<{
  label: string;
  children: ReactNode;
  href?: string;
  icon?: string;
  className?: string;
}>) {
  const body = (
    <>
      {icon && (
        <span className="meta-chip-icon" aria-hidden="true">
          {icon}
        </span>
      )}
      <span className="meta-chip-value">{children}</span>
      {href && (
        <span className="meta-chip-ext" aria-hidden="true">
          {"↗"}
        </span>
      )}
    </>
  );
  if (href) {
    return (
      <Tooltip label={label} className={className}>
        <a
          className="meta-chip meta-chip-link"
          href={href}
          target="_blank"
          rel="noreferrer"
          aria-label={label}
        >
          {body}
        </a>
      </Tooltip>
    );
  }
  return (
    <Tooltip label={label} className={className}>
      <span className="meta-chip">
        <span className="visually-hidden">{label}: </span>
        {body}
      </span>
    </Tooltip>
  );
}

export type LegendItem = { glyph: ReactNode; label: string };

/** One quiet row explaining the section's symbols to first-time visitors. */
export function Legend({ items }: Readonly<{ items: LegendItem[] }>) {
  return (
    <ul className="legend" aria-label="Legend">
      {items.map((item) => (
        <li key={item.label}>
          <span className="legend-glyph" aria-hidden="true">
            {item.glyph}
          </span>
          <span>{item.label}</span>
        </li>
      ))}
    </ul>
  );
}

const clampThreshold = 260;

/**
 * Long prose collapses to a few lines with a "More" toggle so cards stay
 * scannable without hiding anything permanently.
 */
export function ClampText({
  text,
  className = "",
  lines = 3,
}: Readonly<{ text: string; className?: string; lines?: 2 | 3 | 4 }>) {
  const [expanded, setExpanded] = useState(false);
  const clampable = text.length > clampThreshold;
  const clampClass = clampable && !expanded ? `clamp clamp-${lines}` : "";
  return (
    <div className={`clamp-block ${className}`}>
      <p className={clampClass}>{text}</p>
      {clampable && (
        <button
          type="button"
          className="clamp-toggle"
          onClick={() => setExpanded((current) => !current)}
          aria-expanded={expanded}
        >
          {expanded ? "Less" : "More"}
        </button>
      )}
    </div>
  );
}
