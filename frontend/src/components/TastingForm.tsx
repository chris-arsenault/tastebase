import { useCallback, useEffect, type RefObject, type SubmitEvent } from "react";
import { useCamera, type CameraControls } from "../hooks/useCamera";
import { useRecorder, type RecorderControls } from "../hooks/useRecorder";
import { PepperSelector, ScoreSelector } from "./display";
import type { FormState } from "../hooks/useTastings";
import type { ProductType, TastingRecord } from "../types";

function MediaSlot({ label, icon, camera, videoRef, small }: Readonly<{
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
        <button type="button" className="media-remove" onClick={camera.clear}>{"\u00D7"}</button>
      </div>
    );
  }
  if (camera.active) {
    return (
      <div className={slotClass}>
        <div className="media-camera">
          <video ref={videoRef} playsInline muted autoPlay />
          <button type="button" onClick={camera.capture}>{small ? "\uD83D\uDCF8" : "\uD83D\uDCF8 Capture"}</button>
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

function VoiceCapture({ recorder }: Readonly<{ recorder: RecorderControls }>) {
  if (recorder.audioUrl) {
    return (
      <div className="voice-capture">
        <div className="voice-preview">
          {/* eslint-disable-next-line jsx-a11y/media-has-caption -- user-recorded tasting notes, no captions available */}
          <audio controls src={recorder.audioUrl} />
          <button type="button" onClick={recorder.clear}>Remove</button>
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
          <button type="button" onClick={recorder.stop}>Stop</button>
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

function DetailsSection({ form, setForm }: Readonly<{
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
            <input value={form.name} onChange={(e) => setForm((p) => ({ ...p, name: e.target.value }))} placeholder="Product name" />
          </label>
          <label className="form-field">
            <span>Maker</span>
            <input value={form.maker} onChange={(e) => setForm((p) => ({ ...p, maker: e.target.value }))} placeholder="Brand" />
          </label>
        </div>
        <div className="form-row">
          <label className="form-field">
            <span>Style</span>
            <input value={form.style} onChange={(e) => setForm((p) => ({ ...p, style: e.target.value }))} placeholder="e.g. Habanero" />
          </label>
          <label className="form-field">
            <span>Date</span>
            <input type="date" value={form.date} onChange={(e) => setForm((p) => ({ ...p, date: e.target.value }))} />
          </label>
          <label className="form-field">
            <span>URL</span>
            <input type="url" value={form.productUrl} onChange={(e) => setForm((p) => ({ ...p, productUrl: e.target.value }))} placeholder="https://..." />
          </label>
        </div>
      </div>
    </section>
  );
}

function RatingsSection({ form, setForm, formProductType }: Readonly<{
  form: FormState;
  setForm: React.Dispatch<React.SetStateAction<FormState>>;
  formProductType: string;
}>) {
  const onScoreChange = useCallback((v: string) => setForm((p) => ({ ...p, score: v })), [setForm]);
  const onHeatUserChange = useCallback((v: string) => setForm((p) => ({ ...p, heatUser: v })), [setForm]);
  const onHeatVendorChange = useCallback((v: string) => setForm((p) => ({ ...p, heatVendor: v })), [setForm]);
  return (
    <section className="form-section form-ratings">
      <h3>Ratings</h3>
      <div className="rating-row">
        <div className="rating-block">
          <span className="rating-label">Score</span>
          <ScoreSelector value={form.score} onChange={onScoreChange} showLabel={false} />
        </div>
        {formProductType !== "drink" && (
          <>
            <div className="rating-block">
              <span className="rating-label">Your Heat</span>
              <PepperSelector value={form.heatUser} onChange={onHeatUserChange} showLabel={false} />
            </div>
            <div className="rating-block">
              <span className="rating-label">Vendor Heat</span>
              <PepperSelector value={form.heatVendor} onChange={onHeatVendorChange} showLabel={false} />
            </div>
          </>
        )}
      </div>
    </section>
  );
}

function NotesSection({ form, setForm }: Readonly<{
  form: FormState;
  setForm: React.Dispatch<React.SetStateAction<FormState>>;
}>) {
  return (
    <section className="form-section">
      <h3>Notes</h3>
      <div className="form-notes">
        <label>
          <span>Your Tasting Notes</span>
          <textarea rows={2} value={form.tastingNotesUser} onChange={(e) => setForm((p) => ({ ...p, tastingNotesUser: e.target.value }))} placeholder="Flavor, impressions..." />
        </label>
        <label>
          <span>Vendor Description</span>
          <textarea rows={2} value={form.tastingNotesVendor} onChange={(e) => setForm((p) => ({ ...p, tastingNotesVendor: e.target.value }))} placeholder="Official description..." />
        </label>
      </div>
    </section>
  );
}

function FormFooter({ submitStatus, canSubmit, onClose }: Readonly<{
  submitStatus: "idle" | "saving" | "saved";
  canSubmit: boolean;
  onClose: () => void;
}>) {
  return (
    <footer className="form-footer">
      <button type="button" className="btn-cancel" onClick={onClose}>Cancel</button>
      <button type="submit" className="btn-submit" disabled={!canSubmit}>
        {submitStatus === "saving" ? "Saving..." : "Save"}
      </button>
    </footer>
  );
}

function FormFieldSections({ form, setForm, formProductType }: Readonly<{
  form: FormState;
  setForm: React.Dispatch<React.SetStateAction<FormState>>;
  formProductType: string;
}>) {
  return (
    <>
      <DetailsSection form={form} setForm={setForm} />
      <RatingsSection form={form} setForm={setForm} formProductType={formProductType} />
      <NotesSection form={form} setForm={setForm} />
    </>
  );
}

function MediaSection({ formMode, mediaExpanded, productCamera, ingredientsCamera, nutritionCamera, productRef, ingredientsRef, nutritionRef, recorder }: Readonly<{
  formMode: "add" | "edit";
  mediaExpanded: ToggleState;
  productCamera: CameraControls;
  ingredientsCamera: CameraControls;
  nutritionCamera: CameraControls;
  productRef: RefObject<HTMLVideoElement | null>;
  ingredientsRef: RefObject<HTMLVideoElement | null>;
  nutritionRef: RefObject<HTMLVideoElement | null>;
  recorder: RecorderControls;
}>) {
  return (
    <section className="form-section">
      <div className="form-section-header">
        <h3>{"\uD83D\uDCF7"} Photos</h3>
        {formMode === "edit" && (
          <button type="button" className="form-section-toggle" onClick={() => mediaExpanded.set(!mediaExpanded.value)}>
            {mediaExpanded.value ? "Hide" : "Edit"}
          </button>
        )}
      </div>
      {(formMode === "add" || mediaExpanded.value) && (
        <div className="media-grid">
          <MediaSlot label="Product" icon={"\uD83D\uDCF7"} camera={productCamera} videoRef={productRef} />
          <MediaSlot label="Ingredients" icon={"\uD83D\uDCCB"} camera={ingredientsCamera} videoRef={ingredientsRef} small />
          <MediaSlot label="Nutrition" icon={"\uD83D\uDCCA"} camera={nutritionCamera} videoRef={nutritionRef} small />
        </div>
      )}
      {formMode === "add" && <VoiceCapture recorder={recorder} />}
    </section>
  );
}

const hasAnyMedia = (cameras: CameraControls[], audioBase64: string) =>
  cameras.some((c) => Boolean(c.base64)) || Boolean(audioBase64);

const resolveProductType = (record: TastingRecord | null, formMode: string, productType: ProductType | "all") =>
  record?.productType ?? (formMode === "add" ? productType : "sauce");

type ToggleState = {
  value: boolean;
  set: (v: boolean) => void;
};

type TastingFormProps = {
  formMode: "add" | "edit";
  form: FormState;
  setForm: React.Dispatch<React.SetStateAction<FormState>>;
  manualFields: ToggleState;
  mediaExpanded: ToggleState;
  submitStatus: "idle" | "saving" | "saved";
  viewingRecord: TastingRecord | null;
  productType: ProductType | "all";
  onSubmit: (mediaData: {
    imageBase64: string;
    imageMimeType: string;
    ingredientsImageBase64: string;
    ingredientsImageMimeType: string;
    nutritionImageBase64: string;
    nutritionImageMimeType: string;
    audioBase64: string;
    audioMimeType: string;
  }) => void;
  onClose: () => void;
  onError: (msg: string) => void;
};

export function TastingForm({
  formMode,
  form,
  setForm,
  manualFields,
  mediaExpanded,
  submitStatus,
  viewingRecord,
  productType,
  onSubmit,
  onClose,
  onError
}: Readonly<TastingFormProps>) {
  const [productRef, productCamera] = useCamera(onError);
  const [ingredientsRef, ingredientsCamera] = useCamera(onError);
  const [nutritionRef, nutritionCamera] = useCamera(onError);
  const recorder = useRecorder(onError);

  useEffect(() => {
    if (formMode !== "edit" || !viewingRecord) return;
    productCamera.setExistingPreview(viewingRecord.imageUrl ?? "");
    ingredientsCamera.setExistingPreview(viewingRecord.ingredientsImageUrl ?? "");
    nutritionCamera.setExistingPreview(viewingRecord.nutritionImageUrl ?? "");
  }, [formMode, viewingRecord]); // eslint-disable-line react-hooks/exhaustive-deps

  useEffect(() => () => {
    productCamera.stop();
    ingredientsCamera.stop();
    nutritionCamera.stop();
    recorder.stop();
  }, []); // eslint-disable-line react-hooks/exhaustive-deps

  const handleFormSubmit = (event: SubmitEvent) => {
    event.preventDefault();
    onSubmit({
      imageBase64: productCamera.base64,
      imageMimeType: productCamera.mimeType,
      ingredientsImageBase64: ingredientsCamera.base64,
      ingredientsImageMimeType: ingredientsCamera.mimeType,
      nutritionImageBase64: nutritionCamera.base64,
      nutritionImageMimeType: nutritionCamera.mimeType,
      audioBase64: recorder.audioBase64,
      audioMimeType: recorder.audioMimeType
    });
  };

  const showFields = formMode === "edit" || manualFields.value;
  const formProductType = resolveProductType(viewingRecord, formMode, productType);
  const hasMedia = hasAnyMedia([productCamera, ingredientsCamera, nutritionCamera], recorder.audioBase64);
  const canSubmit = submitStatus !== "saving" && (showFields || hasMedia);

  return (
    <div className="form-overlay">
      <section className="form-modal" role="dialog" aria-modal="true">
        <header className="form-header">
          <h2>{formMode === "edit" ? "Edit Tasting" : "New Tasting"}</h2>
          <button type="button" className="form-close" onClick={onClose}>{"\u00D7"}</button>
        </header>

        <form className="form-body" onSubmit={handleFormSubmit}>
          <MediaSection
            formMode={formMode} mediaExpanded={mediaExpanded}
            productCamera={productCamera} ingredientsCamera={ingredientsCamera} nutritionCamera={nutritionCamera}
            productRef={productRef} ingredientsRef={ingredientsRef} nutritionRef={nutritionRef}
            recorder={recorder}
          />
          {showFields && <FormFieldSections form={form} setForm={setForm} formProductType={formProductType} />}
          {!showFields && (
            <button type="button" className="form-toggle-manual" onClick={() => manualFields.set(true)}>
              + Add details manually
            </button>
          )}
          <FormFooter submitStatus={submitStatus} canSubmit={canSubmit} onClose={onClose} />
        </form>
      </section>
    </div>
  );
}
