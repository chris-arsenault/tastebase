import { useEffect, useState } from "react";
import { createTasting, deleteTasting as apiDeleteTasting, fetchTastings, rerunTasting, updateTastingMedia } from "../api";
import type { AuthState } from "./useAuth";
import type { CreateTastingInput, TastingRecord, UpdateTastingMediaInput } from "../types";

type FormMode = "add" | "edit";

const toNumberOrNull = (value: string) => {
  if (!value.trim()) return null;
  const parsed = Number(value);
  return Number.isNaN(parsed) ? null : parsed;
};

const trimOrUndefined = (value: string) => value.trim() || undefined;

const emptyForm = {
  name: "",
  maker: "",
  date: "",
  score: "",
  style: "",
  heatUser: "",
  heatVendor: "",
  tastingNotesUser: "",
  tastingNotesVendor: "",
  productUrl: ""
};

export type FormState = typeof emptyForm;

const numToString = (value: number | null | undefined) => value != null ? String(value) : "";

const recordToFormState = (record: TastingRecord): FormState => ({
  name: record.name || "",
  maker: record.maker || "",
  date: record.date || "",
  score: numToString(record.score),
  style: record.style || "",
  heatUser: numToString(record.heatUser),
  heatVendor: numToString(record.heatVendor),
  tastingNotesUser: record.tastingNotesUser || "",
  tastingNotesVendor: record.tastingNotesVendor || "",
  productUrl: record.productUrl || ""
});

type MediaData = {
  imageBase64: string;
  imageMimeType: string;
  ingredientsImageBase64: string;
  ingredientsImageMimeType: string;
  nutritionImageBase64: string;
  nutritionImageMimeType: string;
  audioBase64: string;
  audioMimeType: string;
};

const trimFallback = (value: string, fallback: string) => value.trim() || fallback;
const numFallback = (value: string, fallback: number | null) => toNumberOrNull(value) ?? fallback;

const buildEditedRecord = (existing: TastingRecord, base: TastingRecord, formData: FormState): TastingRecord => ({
  ...base,
  name: trimFallback(formData.name, existing.name),
  maker: trimFallback(formData.maker, existing.maker),
  date: formData.date || existing.date,
  score: numFallback(formData.score, existing.score),
  style: trimFallback(formData.style, existing.style),
  heatUser: numFallback(formData.heatUser, existing.heatUser),
  heatVendor: numFallback(formData.heatVendor, existing.heatVendor),
  tastingNotesUser: trimFallback(formData.tastingNotesUser, existing.tastingNotesUser),
  tastingNotesVendor: trimFallback(formData.tastingNotesVendor, existing.tastingNotesVendor),
  productUrl: trimFallback(formData.productUrl, existing.productUrl),
  needsAttention: false,
  attentionReason: undefined
});

const buildMediaPayload = (mediaData: MediaData): UpdateTastingMediaInput | null => {
  const hasMedia = Boolean(mediaData.imageBase64 || mediaData.ingredientsImageBase64 || mediaData.nutritionImageBase64);
  if (!hasMedia) return null;
  return {
    imageBase64: mediaData.imageBase64 || undefined,
    imageMimeType: mediaData.imageMimeType || undefined,
    ingredientsImageBase64: mediaData.ingredientsImageBase64 || undefined,
    ingredientsImageMimeType: mediaData.ingredientsImageMimeType || undefined,
    nutritionImageBase64: mediaData.nutritionImageBase64 || undefined,
    nutritionImageMimeType: mediaData.nutritionImageMimeType || undefined
  };
};

const buildCreatePayload = (formData: FormState, mediaData: MediaData): CreateTastingInput => ({
  name: trimOrUndefined(formData.name),
  maker: trimOrUndefined(formData.maker),
  date: formData.date || undefined,
  score: toNumberOrNull(formData.score),
  style: trimOrUndefined(formData.style),
  heatUser: toNumberOrNull(formData.heatUser),
  heatVendor: toNumberOrNull(formData.heatVendor),
  tastingNotesUser: trimOrUndefined(formData.tastingNotesUser),
  tastingNotesVendor: trimOrUndefined(formData.tastingNotesVendor),
  productUrl: trimOrUndefined(formData.productUrl),
  imageBase64: mediaData.imageBase64 || undefined,
  imageMimeType: mediaData.imageMimeType || undefined,
  ingredientsImageBase64: mediaData.ingredientsImageBase64 || undefined,
  ingredientsImageMimeType: mediaData.ingredientsImageMimeType || undefined,
  nutritionImageBase64: mediaData.nutritionImageBase64 || undefined,
  nutritionImageMimeType: mediaData.nutritionImageMimeType || undefined,
  voiceBase64: mediaData.audioBase64 || undefined,
  voiceMimeType: mediaData.audioMimeType || undefined
});

const asError = (error: unknown) => (error as Error).message;

function useFormState() {
  const [formOpen, setFormOpen] = useState(false);
  const [formMode, setFormMode] = useState<FormMode>("add");
  const [editingId, setEditingId] = useState<string | null>(null);
  const [viewingRecord, setViewingRecord] = useState<TastingRecord | null>(null);
  const [form, setForm] = useState({ ...emptyForm });
  const [showManualFields, setShowManualFields] = useState(false);
  const [mediaExpanded, setMediaExpanded] = useState(true);
  const [viewOpen, setViewOpen] = useState(false);

  const openAddForm = () => {
    setFormMode("add");
    setEditingId(null);
    setForm({ ...emptyForm });
    setShowManualFields(false);
    setMediaExpanded(true);
    setViewOpen(false);
    setViewingRecord(null);
    setFormOpen(true);
  };

  const openEditForm = (record: TastingRecord) => {
    setFormMode("edit");
    setEditingId(record.id);
    setViewingRecord(record);
    setMediaExpanded(false);
    setViewOpen(false);
    setForm(recordToFormState(record));
    setShowManualFields(true);
    setFormOpen(true);
  };

  const closeForm = () => {
    setFormOpen(false);
    setFormMode("add");
    setEditingId(null);
    setViewingRecord(null);
    setShowManualFields(false);
    setMediaExpanded(true);
  };

  const openViewModal = (record: TastingRecord) => {
    setViewingRecord(record);
    setViewOpen(true);
  };

  const closeViewModal = () => {
    setViewOpen(false);
    setViewingRecord(null);
  };

  return {
    formOpen, formMode, editingId, viewingRecord, form, setForm,
    showManualFields, setShowManualFields, mediaExpanded, setMediaExpanded,
    viewOpen, openAddForm, openEditForm, closeForm, openViewModal, closeViewModal
  };
}

function useInitialFetch() {
  const [tastings, setTastings] = useState<TastingRecord[]>([]);
  const [loading, setLoading] = useState(true);
  const [errorMessage, setErrorMessage] = useState("");
  useEffect(() => {
    let stale = false;
    fetchTastings()
      .then((data) => { if (!stale) setTastings(data); })
      .catch((e: unknown) => { if (!stale) setErrorMessage(asError(e)); })
      .finally(() => { if (!stale) setLoading(false); });
    return () => { stale = true; };
  }, []);
  return { tastings, setTastings, loading, setLoading, errorMessage, setErrorMessage };
}

export function useTastings(auth: AuthState) {
  const { tastings, setTastings, loading, setLoading, errorMessage, setErrorMessage } = useInitialFetch();
  const [submitStatus, setSubmitStatus] = useState<"idle" | "saving" | "saved">("idle");
  const [rerunId, setRerunId] = useState<string | null>(null);
  const [deleteTarget, setDeleteTarget] = useState<TastingRecord | null>(null);
  const [deleteStatus, setDeleteStatus] = useState<"idle" | "deleting">("idle");
  const formState = useFormState();

  useEffect(() => {
    if (auth.status === "signedOut") { formState.closeForm(); formState.closeViewModal(); }
  }, [auth.status]); // eslint-disable-line react-hooks/exhaustive-deps -- only react to auth changes

  const updateTasting = (id: string, updater: (t: TastingRecord) => TastingRecord) =>
    setTastings((prev) => prev.map((t) => (t.id === id ? updater(t) : t)));

  const removeTasting = (id: string) =>
    setTastings((prev) => prev.filter((t) => t.id !== id));

  const loadTastings = (showLoading = false) => {
    if (showLoading) setLoading(true);
    fetchTastings().then(setTastings).catch((e: unknown) => setErrorMessage(asError(e))).finally(() => setLoading(false));
  };

  const handleSubmit = (mediaData: MediaData) => {
    if (auth.status !== "signedIn") { setErrorMessage("Sign in to save."); return; }
    setSubmitStatus("saving");
    setErrorMessage("");
    const op = formState.formMode === "edit" && formState.editingId
      ? performEdit(formState.editingId, formState.form, mediaData, auth.token)
      : performCreate(buildCreatePayload(formState.form, mediaData), auth.token);
    op.catch((e: unknown) => setErrorMessage(asError(e)))
      .finally(() => setSubmitStatus("idle"));
  };

  const finishSubmit = () => { setSubmitStatus("saved"); formState.closeForm(); };

  const performEdit = async (id: string, formData: FormState, mediaData: MediaData, token: string) => {
    const mediaPayload = buildMediaPayload(mediaData);
    const updatedMedia = mediaPayload ? await updateTastingMedia(id, mediaPayload, token) : null;
    updateTasting(id, (t) => buildEditedRecord(t, updatedMedia ?? t, formData));
    finishSubmit();
  };

  const performCreate = async (payload: CreateTastingInput, token: string) => {
    await createTasting(payload, token);
    loadTastings(true);
    finishSubmit();
  };

  const handleRerun = (record: TastingRecord) => {
    if (auth.status !== "signedIn") { setErrorMessage("Sign in to rerun."); return; }
    setErrorMessage("");
    setRerunId(record.id);
    rerunTasting(record.id, auth.token)
      .then(() => {
        updateTasting(record.id, (item) => ({ ...item, status: "pending", processingError: undefined }));
        loadTastings(true);
      })
      .catch((e: unknown) => setErrorMessage(asError(e)))
      .finally(() => setRerunId(null));
  };

  const openDeleteModal = (record: TastingRecord) => {
    if (auth.status !== "signedIn") { setErrorMessage("Sign in to delete."); return; }
    setDeleteTarget(record);
  };

  const closeDeleteModal = () => { setDeleteTarget(null); setDeleteStatus("idle"); };

  const confirmDelete = () => {
    if (!deleteTarget || auth.status !== "signedIn") return;
    setErrorMessage("");
    setDeleteStatus("deleting");
    apiDeleteTasting(deleteTarget.id, auth.token)
      .then(() => { removeTasting(deleteTarget.id); closeDeleteModal(); })
      .catch((e: unknown) => { setErrorMessage(asError(e)); setDeleteStatus("idle"); });
  };

  return {
    tastings, loading, errorMessage, setErrorMessage, submitStatus, rerunId,
    deleteTarget, deleteStatus, ...formState,
    handleSubmit, handleRerun, openDeleteModal, closeDeleteModal, confirmDelete
  };
}
