import { useEffect, type RefObject, type SubmitEvent } from "react";
import { useCamera, type CameraControls } from "../hooks/useCamera";
import { useRecorder, type RecorderControls } from "../hooks/useRecorder";
import type { FormState } from "../hooks/useTastings";
import type { ProductType, TastingRecord } from "../types";
import {
  MediaSlot,
  VoiceCapture,
  DetailsSection,
  RatingsSection,
  NotesSection,
  FormFooter,
} from "./TastingFormSections";

function useTastingMedia(
  onError: (msg: string) => void,
  formMode: "add" | "edit",
  viewingRecord: TastingRecord | null,
) {
  const [productRef, productCamera] = useCamera(onError);
  const [ingredientsRef, ingredientsCamera] = useCamera(onError);
  const [nutritionRef, nutritionCamera] = useCamera(onError);
  const recorder = useRecorder(onError);

  useEffect(() => {
    if (formMode !== "edit" || !viewingRecord) return;
    productCamera.setExistingPreview(viewingRecord.imageUrl ?? "");
    ingredientsCamera.setExistingPreview(
      viewingRecord.ingredientsImageUrl ?? "",
    );
    nutritionCamera.setExistingPreview(viewingRecord.nutritionImageUrl ?? "");
  }, [formMode, viewingRecord]); // eslint-disable-line react-hooks/exhaustive-deps

  /* eslint-disable react-hooks/exhaustive-deps -- cleanup only, intentionally empty deps */
  useEffect(
    () => () => {
      productCamera.stop();
      ingredientsCamera.stop();
      nutritionCamera.stop();
      recorder.stop();
    },
    [],
  );
  /* eslint-enable react-hooks/exhaustive-deps */

  const collectMediaData = () => ({
    imageBase64: productCamera.base64,
    imageMimeType: productCamera.mimeType,
    ingredientsImageBase64: ingredientsCamera.base64,
    ingredientsImageMimeType: ingredientsCamera.mimeType,
    nutritionImageBase64: nutritionCamera.base64,
    nutritionImageMimeType: nutritionCamera.mimeType,
    audioBlob: recorder.audioBlob,
    audioMimeType: recorder.audioMimeType,
  });

  const hasMedia =
    [productCamera, ingredientsCamera, nutritionCamera].some((c) =>
      Boolean(c.base64),
    ) || Boolean(recorder.audioBlob);

  return {
    productRef,
    productCamera,
    ingredientsRef,
    ingredientsCamera,
    nutritionRef,
    nutritionCamera,
    recorder,
    collectMediaData,
    hasMedia,
  };
}

type ToggleState = {
  value: boolean;
  set: (v: boolean) => void;
};

function FormFieldSections({
  form,
  setForm,
  formProductType,
}: Readonly<{
  form: FormState;
  setForm: React.Dispatch<React.SetStateAction<FormState>>;
  formProductType: string;
}>) {
  return (
    <>
      <DetailsSection form={form} setForm={setForm} />
      <RatingsSection
        form={form}
        setForm={setForm}
        formProductType={formProductType}
      />
      <NotesSection form={form} setForm={setForm} />
    </>
  );
}

function MediaSection({
  formMode,
  mediaExpanded,
  productCamera,
  ingredientsCamera,
  nutritionCamera,
  productRef,
  ingredientsRef,
  nutritionRef,
  recorder,
}: Readonly<{
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
          <button
            type="button"
            className="form-section-toggle"
            onClick={() => mediaExpanded.set(!mediaExpanded.value)}
          >
            {mediaExpanded.value ? "Hide" : "Edit"}
          </button>
        )}
      </div>
      {(formMode === "add" || mediaExpanded.value) && (
        <div className="media-grid">
          <MediaSlot
            label="Product"
            icon={"\uD83D\uDCF7"}
            camera={productCamera}
            videoRef={productRef}
          />
          <MediaSlot
            label="Ingredients"
            icon={"\uD83D\uDCCB"}
            camera={ingredientsCamera}
            videoRef={ingredientsRef}
            small
          />
          <MediaSlot
            label="Nutrition"
            icon={"\uD83D\uDCCA"}
            camera={nutritionCamera}
            videoRef={nutritionRef}
            small
          />
        </div>
      )}
      {formMode === "add" && <VoiceCapture recorder={recorder} />}
    </section>
  );
}

const resolveProductType = (
  record: TastingRecord | null,
  formMode: string,
  productType: ProductType | "all",
) => record?.productType ?? (formMode === "add" ? productType : "sauce");

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
    audioBlob: Blob | null;
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
  onError,
}: Readonly<TastingFormProps>) {
  const media = useTastingMedia(onError, formMode, viewingRecord);
  const showFields = formMode === "edit" || manualFields.value;
  const formProductType = resolveProductType(
    viewingRecord,
    formMode,
    productType,
  );
  const canSubmit = submitStatus !== "saving" && (showFields || media.hasMedia);

  const handleFormSubmit = (event: SubmitEvent) => {
    event.preventDefault();
    onSubmit(media.collectMediaData());
  };

  return (
    <div className="form-overlay">
      <section className="form-modal" role="dialog" aria-modal="true">
        <header className="form-header">
          <h2>{formMode === "edit" ? "Edit Tasting" : "New Tasting"}</h2>
          <button type="button" className="form-close" onClick={onClose}>
            {"\u00D7"}
          </button>
        </header>
        <form className="form-body" onSubmit={handleFormSubmit}>
          <MediaSection
            formMode={formMode}
            mediaExpanded={mediaExpanded}
            productCamera={media.productCamera}
            ingredientsCamera={media.ingredientsCamera}
            nutritionCamera={media.nutritionCamera}
            productRef={media.productRef}
            ingredientsRef={media.ingredientsRef}
            nutritionRef={media.nutritionRef}
            recorder={media.recorder}
          />
          {showFields && (
            <FormFieldSections
              form={form}
              setForm={setForm}
              formProductType={formProductType}
            />
          )}
          {!showFields && (
            <button
              type="button"
              className="form-toggle-manual"
              onClick={() => manualFields.set(true)}
            >
              + Add details manually
            </button>
          )}
          <FormFooter
            submitStatus={submitStatus}
            canSubmit={canSubmit}
            onClose={onClose}
          />
        </form>
      </section>
    </div>
  );
}
