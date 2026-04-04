import { useCallback, type RefObject } from "react";
import type { CameraControls } from "../hooks/useCamera";
import type { RecorderControls } from "../hooks/useRecorder";
import { PepperSelector, ScoreSelector } from "./display";
import type { FormState } from "../hooks/useTastings";

export function MediaSlot({
  label,
  icon,
  camera,
  videoRef,
  small,
}: Readonly<{
  label: string;
  icon: string;
  camera: CameraControls;
  videoRef: RefObject<HTMLVideoElement | null>;
  small?: boolean;
}>) {
  const slotClass = `media-slot${small ? " media-slot-sm" : ""} ${camera.preview ? "has-image" : ""}`;
  if (camera.preview) {
    return (
      <div className={slotClass}>
        <img src={camera.preview} alt={label} />
        <button type="button" className="media-remove" onClick={camera.clear}>
          {"\u00D7"}
        </button>
      </div>
    );
  }
  if (camera.active) {
    return (
      <div className={slotClass}>
        <div className="media-camera">
          <video ref={videoRef} playsInline muted autoPlay />
          <button type="button" onClick={camera.capture}>
            {small ? "\uD83D\uDCF8" : "\uD83D\uDCF8 Capture"}
          </button>
        </div>
      </div>
    );
  }
  return (
    <div className={slotClass}>
      <button type="button" className="media-add" onClick={camera.start}>
        <span>{icon}</span>
        <small>{label}</small>
      </button>
    </div>
  );
}

export function VoiceCapture({
  recorder,
}: Readonly<{ recorder: RecorderControls }>) {
  if (recorder.audioUrl) {
    return (
      <div className="voice-capture">
        <div className="voice-preview">
          {/* eslint-disable-next-line jsx-a11y/media-has-caption -- user-recorded tasting notes, no captions available */}
          <audio controls src={recorder.audioUrl} />
          <button type="button" onClick={recorder.clear}>
            Remove
          </button>
        </div>
      </div>
    );
  }
  if (recorder.isRecording) {
    return (
      <div className="voice-capture">
        <div className="voice-recording">
          <span className="voice-pulse" />
          <span>Recording...</span>
          <button type="button" onClick={recorder.stop}>
            Stop
          </button>
        </div>
      </div>
    );
  }
  return (
    <div className="voice-capture">
      <button type="button" className="voice-start" onClick={recorder.start}>
        {"\uD83C\uDF99\uFE0F"} Record tasting notes
      </button>
    </div>
  );
}

export function DetailsSection({
  form,
  setForm,
}: Readonly<{
  form: FormState;
  setForm: React.Dispatch<React.SetStateAction<FormState>>;
}>) {
  return (
    <section className="form-section">
      <h3>Details</h3>
      <div className="form-fields">
        <div className="form-row">
          <label className="form-field form-field-lg">
            <span>Name</span>
            <input
              value={form.name}
              onChange={(e) => setForm((p) => ({ ...p, name: e.target.value }))}
              placeholder="Product name"
            />
          </label>
          <label className="form-field">
            <span>Maker</span>
            <input
              value={form.maker}
              onChange={(e) =>
                setForm((p) => ({ ...p, maker: e.target.value }))
              }
              placeholder="Brand"
            />
          </label>
        </div>
        <div className="form-row">
          <label className="form-field">
            <span>Style</span>
            <input
              value={form.style}
              onChange={(e) =>
                setForm((p) => ({ ...p, style: e.target.value }))
              }
              placeholder="e.g. Habanero"
            />
          </label>
          <label className="form-field">
            <span>Date</span>
            <input
              type="date"
              value={form.date}
              onChange={(e) => setForm((p) => ({ ...p, date: e.target.value }))}
            />
          </label>
          <label className="form-field">
            <span>URL</span>
            <input
              type="url"
              value={form.productUrl}
              onChange={(e) =>
                setForm((p) => ({ ...p, productUrl: e.target.value }))
              }
              placeholder="https://..."
            />
          </label>
        </div>
      </div>
    </section>
  );
}

export function RatingsSection({
  form,
  setForm,
  formProductType,
}: Readonly<{
  form: FormState;
  setForm: React.Dispatch<React.SetStateAction<FormState>>;
  formProductType: string;
}>) {
  const onScoreChange = useCallback(
    (v: string) => setForm((p) => ({ ...p, score: v })),
    [setForm],
  );
  const onHeatUserChange = useCallback(
    (v: string) => setForm((p) => ({ ...p, heatUser: v })),
    [setForm],
  );
  const onHeatVendorChange = useCallback(
    (v: string) => setForm((p) => ({ ...p, heatVendor: v })),
    [setForm],
  );
  return (
    <section className="form-section form-ratings">
      <h3>Ratings</h3>
      <div className="rating-row">
        <div className="rating-block">
          <span className="rating-label">Score</span>
          <ScoreSelector
            value={form.score}
            onChange={onScoreChange}
            showLabel={false}
          />
        </div>
        {formProductType !== "drink" && (
          <>
            <div className="rating-block">
              <span className="rating-label">Your Heat</span>
              <PepperSelector
                value={form.heatUser}
                onChange={onHeatUserChange}
                showLabel={false}
              />
            </div>
            <div className="rating-block">
              <span className="rating-label">Vendor Heat</span>
              <PepperSelector
                value={form.heatVendor}
                onChange={onHeatVendorChange}
                showLabel={false}
              />
            </div>
          </>
        )}
      </div>
    </section>
  );
}

export function NotesSection({
  form,
  setForm,
}: Readonly<{
  form: FormState;
  setForm: React.Dispatch<React.SetStateAction<FormState>>;
}>) {
  return (
    <section className="form-section">
      <h3>Notes</h3>
      <div className="form-notes">
        <label>
          <span>Your Tasting Notes</span>
          <textarea
            rows={2}
            value={form.tastingNotesUser}
            onChange={(e) =>
              setForm((p) => ({ ...p, tastingNotesUser: e.target.value }))
            }
            placeholder="Flavor, impressions..."
          />
        </label>
        <label>
          <span>Vendor Description</span>
          <textarea
            rows={2}
            value={form.tastingNotesVendor}
            onChange={(e) =>
              setForm((p) => ({ ...p, tastingNotesVendor: e.target.value }))
            }
            placeholder="Official description..."
          />
        </label>
      </div>
    </section>
  );
}

export function FormFooter({
  submitStatus,
  canSubmit,
  onClose,
}: Readonly<{
  submitStatus: "idle" | "saving" | "saved";
  canSubmit: boolean;
  onClose: () => void;
}>) {
  return (
    <footer className="form-footer">
      <button type="button" className="btn-cancel" onClick={onClose}>
        Cancel
      </button>
      <button type="submit" className="btn-submit" disabled={!canSubmit}>
        {submitStatus === "saving" ? "Saving..." : "Save"}
      </button>
    </footer>
  );
}
