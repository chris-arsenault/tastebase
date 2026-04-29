import { useEffect, useState } from "react";
import {
  createTasting,
  deleteTasting as apiDeleteTasting,
  fetchTastings,
  rerunTasting,
  updateTastingMedia,
} from "../api";
import type { AuthState } from "./useAuth";
import type { TastingRecord } from "../types";
import {
  buildCreatePayload,
  buildEditMediaPayload,
  uploadAllMedia,
  type MediaData,
} from "./useTastings.media";

type FormMode = "add" | "edit";

const toNumberOrNull = (value: string) => {
  if (!value.trim()) return null;
  const parsed = Number(value);
  return Number.isNaN(parsed) ? null : parsed;
};

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
  productUrl: "",
};

export type FormState = typeof emptyForm;

const numToString = (value: number | null | undefined) =>
  value != null ? String(value) : "";

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
  productUrl: record.productUrl || "",
});

const trimFallback = (value: string, fallback: string) =>
  value.trim() || fallback;
const numFallback = (value: string, fallback: number | null) =>
  toNumberOrNull(value) ?? fallback;

const performEdit = async (
  id: string,
  formData: FormState,
  mediaData: MediaData,
  token: string,
  ops: TastingOps,
) => {
  const keys = await uploadAllMedia(mediaData, token);
  const mediaPayload = buildEditMediaPayload(keys);
  const updatedMedia = mediaPayload
    ? await updateTastingMedia(id, mediaPayload, token)
    : null;
  ops.update(id, (t) => buildEditedRecord(t, updatedMedia ?? t, formData));
};

const performCreate = async (
  formData: FormState,
  mediaData: MediaData,
  token: string,
  ops: TastingOps,
) => {
  const keys = await uploadAllMedia(mediaData, token);
  await createTasting(buildCreatePayload(formData, keys), token);
  ops.reload(true);
};

const buildEditedRecord = (
  existing: TastingRecord,
  base: TastingRecord,
  formData: FormState,
): TastingRecord => ({
  ...base,
  name: trimFallback(formData.name, existing.name),
  maker: trimFallback(formData.maker, existing.maker),
  date: formData.date || existing.date,
  score: numFallback(formData.score, existing.score),
  style: trimFallback(formData.style, existing.style),
  heatUser: numFallback(formData.heatUser, existing.heatUser),
  heatVendor: numFallback(formData.heatVendor, existing.heatVendor),
  tastingNotesUser: trimFallback(
    formData.tastingNotesUser,
    existing.tastingNotesUser,
  ),
  tastingNotesVendor: trimFallback(
    formData.tastingNotesVendor,
    existing.tastingNotesVendor,
  ),
  productUrl: trimFallback(formData.productUrl, existing.productUrl),
  needsAttention: false,
  attentionReason: undefined,
});

const asError = (error: unknown) => (error as Error).message;

function useFormState() {
  const [formOpen, setFormOpen] = useState(false);
  const [formMode, setFormMode] = useState<FormMode>("add");
  const [editingId, setEditingId] = useState<string | null>(null);
  const [viewingRecord, setViewingRecord] = useState<TastingRecord | null>(
    null,
  );
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
    formOpen,
    formMode,
    editingId,
    viewingRecord,
    form,
    setForm,
    showManualFields,
    setShowManualFields,
    mediaExpanded,
    setMediaExpanded,
    viewOpen,
    openAddForm,
    openEditForm,
    closeForm,
    openViewModal,
    closeViewModal,
  };
}

function useInitialFetch() {
  const [tastings, setTastings] = useState<TastingRecord[]>([]);
  const [loading, setLoading] = useState(true);
  const [errorMessage, setErrorMessage] = useState("");
  useEffect(() => {
    let stale = false;
    fetchTastings()
      .then((data) => {
        if (!stale) setTastings(data);
      })
      .catch((e: unknown) => {
        if (!stale) setErrorMessage(asError(e));
      })
      .finally(() => {
        if (!stale) setLoading(false);
      });
    return () => {
      stale = true;
    };
  }, []);
  return {
    tastings,
    setTastings,
    loading,
    setLoading,
    errorMessage,
    setErrorMessage,
  };
}

type TastingOps = {
  update: (id: string, updater: (t: TastingRecord) => TastingRecord) => void;
  remove: (id: string) => void;
  reload: (showLoading?: boolean) => void;
  setError: (msg: string) => void;
};

function useTastingOps(
  setTastings: React.Dispatch<React.SetStateAction<TastingRecord[]>>,
  setLoading: (v: boolean) => void,
  setErrorMessage: (msg: string) => void,
): TastingOps {
  return {
    update: (id, updater) =>
      setTastings((prev) => prev.map((t) => (t.id === id ? updater(t) : t))),
    remove: (id) => setTastings((prev) => prev.filter((t) => t.id !== id)),
    reload: (showLoading = false) => {
      if (showLoading) setLoading(true);
      fetchTastings()
        .then(setTastings)
        .catch((e: unknown) => setErrorMessage(asError(e)))
        .finally(() => setLoading(false));
    },
    setError: setErrorMessage,
  };
}

function useSubmission(
  auth: AuthState,
  formState: ReturnType<typeof useFormState>,
  ops: TastingOps,
) {
  const [submitStatus, setSubmitStatus] = useState<"idle" | "saving" | "saved">(
    "idle",
  );
  const [rerunId, setRerunId] = useState<string | null>(null);

  const finishSubmit = () => {
    setSubmitStatus("saved");
    formState.closeForm();
  };

  const handleSubmit = (mediaData: MediaData) => {
    if (auth.status !== "signedIn") {
      ops.setError("Sign in to save.");
      return;
    }
    setSubmitStatus("saving");
    ops.setError("");
    const op =
      formState.formMode === "edit" && formState.editingId
        ? performEdit(
            formState.editingId,
            formState.form,
            mediaData,
            auth.token,
            ops,
          )
        : performCreate(formState.form, mediaData, auth.token, ops);
    op.then(finishSubmit)
      .catch((e: unknown) => ops.setError(asError(e)))
      .finally(() => setSubmitStatus("idle"));
  };

  const handleRerun = (record: TastingRecord) => {
    if (auth.status !== "signedIn") {
      ops.setError("Sign in to rerun.");
      return;
    }
    ops.setError("");
    setRerunId(record.id);
    rerunTasting(record.id, auth.token)
      .then(() => {
        ops.update(record.id, (item) => ({
          ...item,
          status: "pending",
          processingError: undefined,
        }));
        ops.reload(true);
      })
      .catch((e: unknown) => ops.setError(asError(e)))
      .finally(() => setRerunId(null));
  };

  return { submitStatus, rerunId, handleSubmit, handleRerun };
}

function useDeletion(auth: AuthState, ops: TastingOps) {
  const [deleteTarget, setDeleteTarget] = useState<TastingRecord | null>(null);
  const [deleteStatus, setDeleteStatus] = useState<"idle" | "deleting">("idle");

  const openDeleteModal = (record: TastingRecord) => {
    if (auth.status !== "signedIn") {
      ops.setError("Sign in to delete.");
      return;
    }
    setDeleteTarget(record);
  };

  const closeDeleteModal = () => {
    setDeleteTarget(null);
    setDeleteStatus("idle");
  };

  const confirmDelete = () => {
    if (!deleteTarget || auth.status !== "signedIn") return;
    ops.setError("");
    setDeleteStatus("deleting");
    apiDeleteTasting(deleteTarget.id, auth.token)
      .then(() => {
        ops.remove(deleteTarget.id);
        closeDeleteModal();
      })
      .catch((e: unknown) => {
        ops.setError(asError(e));
        setDeleteStatus("idle");
      });
  };

  return {
    deleteTarget,
    deleteStatus,
    openDeleteModal,
    closeDeleteModal,
    confirmDelete,
  };
}

export function useTastings(auth: AuthState) {
  const {
    tastings,
    setTastings,
    loading,
    setLoading,
    errorMessage,
    setErrorMessage,
  } = useInitialFetch();
  const formState = useFormState();
  const ops = useTastingOps(setTastings, setLoading, setErrorMessage);
  const submission = useSubmission(auth, formState, ops);
  const deletion = useDeletion(auth, ops);

  useEffect(() => {
    if (auth.status === "signedOut") {
      formState.closeForm();
      formState.closeViewModal();
    }
  }, [auth.status]); // eslint-disable-line react-hooks/exhaustive-deps -- only react to auth changes

  return {
    tastings,
    loading,
    errorMessage,
    setErrorMessage,
    ...formState,
    ...submission,
    ...deletion,
  };
}
