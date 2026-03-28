const toInt = (value: number | string, fallback: number) => {
  if (typeof value === "number") return value;
  return value ? parseInt(value, 10) : fallback;
};

const toFloat = (value: number | string, fallback: number) => {
  if (typeof value === "number") return value;
  return value ? parseFloat(value) : fallback;
};

// Compact Heat Slider (1-5 peppers)
export const HeatSlider = ({ value, onChange, label }: { value: string; onChange: (val: string) => void; label?: string }) => {
  const numValue = value ? parseInt(value, 10) : 0;
  return (
    <div className="heat-slider">
      {label && <span className="slider-label">{label}</span>}
      <div className="slider-track heat-track">
        <input
          type="range"
          min="0"
          max="5"
          value={numValue}
          onChange={(e) => onChange(e.target.value === "0" ? "" : e.target.value)}
          className="range-input"
        />
        {/* eslint-disable-next-line local/no-inline-styles -- CSS custom property for dynamic fill width */}
        <div className="slider-fill heat-fill" style={{ '--fill-width': `${(numValue / 5) * 100}%` } as React.CSSProperties} />
        <div className="slider-peppers">
          {[1, 2, 3, 4, 5].map((level) => (
            <span key={level} className={`slider-pepper ${numValue >= level ? "active" : ""}`}>{"\uD83C\uDF36\uFE0F"}</span>
          ))}
        </div>
      </div>
      <span className="slider-value">{numValue > 0 ? numValue : "Any"}</span>
    </div>
  );
};

// Compact Score Slider (0-10)
export const ScoreSlider = ({ value, onChange, label }: { value: string; onChange: (val: string) => void; label?: string }) => {
  const numValue = value ? parseInt(value, 10) : 0;
  const hasValue = value !== "";
  return (
    <div className="score-slider">
      {label && <span className="slider-label">{label}</span>}
      <div className="slider-track score-track">
        <input
          type="range"
          min="0"
          max="10"
          value={numValue}
          onChange={(e) => onChange(e.target.value === "0" ? "" : e.target.value)}
          className="range-input"
        />
        {/* eslint-disable-next-line local/no-inline-styles -- CSS custom property for dynamic fill width */}
        <div className="slider-fill score-fill" style={{ '--fill-width': `${(numValue / 10) * 100}%` } as React.CSSProperties} />
      </div>
      <span className="slider-value">{hasValue && numValue > 0 ? `${numValue}+` : "Any"}</span>
    </div>
  );
};

// Pepper Selector for forms (1-5 scale)
export const PepperSelector = ({ value, onChange, label, showLabel = true }: { value: number | string; onChange: (val: string) => void; label?: string; showLabel?: boolean }) => {
  const numValue = toInt(value, 0);
  return (
    <div className="pepper-selector">
      {showLabel && label && <span className="selector-label">{label}</span>}
      <div className="pepper-row">
        {[1, 2, 3, 4, 5].map((level) => (
          <button
            type="button"
            key={level}
            className={`pepper-btn ${numValue >= level ? "active" : ""}`}
            onClick={() => onChange(numValue === level ? "" : String(level))}
            title={`Heat level ${level}`}
          >
            <span className="pepper">{"\uD83C\uDF36\uFE0F"}</span>
          </button>
        ))}
      </div>
    </div>
  );
};

// Score Selector for forms (0-10 scale)
export const ScoreSelector = ({ value, onChange, label, showLabel = true }: { value: number | string; onChange: (val: string) => void; label?: string; showLabel?: boolean }) => {
  const numValue = toFloat(value, -1);
  return (
    <div className="score-selector">
      {showLabel && label && <span className="selector-label">{label}</span>}
      <div className="score-row">
        {[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10].map((score) => (
          <button
            type="button"
            key={score}
            className={`score-btn ${numValue === score ? "active" : ""} ${numValue > score ? "filled" : ""}`}
            onClick={() => onChange(numValue === score ? "" : String(score))}
            title={`Score ${score}`}
          >
            {score}
          </button>
        ))}
      </div>
    </div>
  );
};

// Display peppers for heat level (read-only)
export const HeatDisplay = ({ value, max = 5 }: { value: number | null; max?: number }) => {
  if (value === null) return <span className="heat-empty">-</span>;
  const filled = Math.min(Math.round(value), max);
  return (
    <span className="heat-display">
      {Array.from({ length: max }, (_, i) => (
        <span key={i} className={`pepper-icon ${i < filled ? "filled" : "empty"}`}>{"\uD83C\uDF36\uFE0F"}</span>
      ))}
    </span>
  );
};

// Display score as visual bar
export const ScoreDisplay = ({ value }: { value: number | null }) => {
  if (value === null) return <span className="score-empty">-</span>;
  const percentage = `${(value / 10) * 100}%`;
  return (
    <div className="score-display">
      <div className="score-bar">
        {/* eslint-disable-next-line local/no-inline-styles -- CSS custom property for dynamic fill width */}
        <div className="score-fill" style={{ '--fill-width': percentage } as React.CSSProperties} />
      </div>
      <span className="score-value">{value.toFixed(1)}</span>
    </div>
  );
};
