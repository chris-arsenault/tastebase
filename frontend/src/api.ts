import { config } from "./config";
import type {
  CreateTastingInput,
  Recipe,
  RecipeFull,
  TastingRecord,
  UpdateTastingMediaInput,
} from "./types";

type TastingUploadType = "image" | "voice";

const dataUrlToBlob = async (dataUrl: string): Promise<Blob> => {
  const response = await fetch(dataUrl);
  return response.blob();
};

export const uploadTastingMedia = async (
  dataUrl: string,
  contentType: string,
  uploadType: TastingUploadType,
  token: string,
): Promise<string> => {
  const presign = await fetch(`${config.apiBaseUrl}/tastings/upload-url`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${token}`,
    },
    body: JSON.stringify({ contentType, uploadType }),
  });
  if (!presign.ok) throw new Error("Failed to get upload URL");
  const { uploadUrl, key } = (await presign.json()) as {
    uploadUrl: string;
    key: string;
    publicUrl: string;
  };
  const blob = await dataUrlToBlob(dataUrl);
  const put = await fetch(uploadUrl, {
    method: "PUT",
    body: blob,
    headers: { "Content-Type": contentType },
  });
  if (!put.ok) throw new Error("Failed to upload media");
  return key;
};

export const fetchTastings = async (): Promise<TastingRecord[]> => {
  const response = await fetch(`${config.apiBaseUrl}/tastings`);
  if (!response.ok) {
    throw new Error("Failed to fetch tastings");
  }
  const payload = (await response.json()) as { data: TastingRecord[] };
  return payload.data ?? [];
};

export const createTasting = async (
  payload: CreateTastingInput,
  token: string,
): Promise<TastingRecord | null> => {
  const response = await fetch(`${config.apiBaseUrl}/tastings`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${token}`,
    },
    body: JSON.stringify(payload),
  });

  if (!response.ok) {
    const errorBody = (await response.json().catch(() => ({}))) as {
      message?: string;
    };
    throw new Error(errorBody.message ?? "Failed to create tasting");
  }

  if (response.status === 204) {
    return null;
  }

  const responseBody = (await response.json()) as { data: TastingRecord };
  return responseBody.data ?? null;
};

export const rerunTasting = async (
  id: string,
  token: string,
): Promise<void> => {
  const response = await fetch(`${config.apiBaseUrl}/tastings/${id}/rerun`, {
    method: "POST",
    headers: {
      Authorization: `Bearer ${token}`,
    },
  });
  if (!response.ok) {
    const errorBody = (await response.json().catch(() => ({}))) as {
      message?: string;
    };
    throw new Error(errorBody.message ?? "Failed to rerun pipeline");
  }
};

export const updateTastingMedia = async (
  id: string,
  payload: UpdateTastingMediaInput,
  token: string,
): Promise<TastingRecord | null> => {
  const response = await fetch(`${config.apiBaseUrl}/tastings/${id}/media`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${token}`,
    },
    body: JSON.stringify(payload),
  });

  if (!response.ok) {
    const errorBody = (await response.json().catch(() => ({}))) as {
      message?: string;
    };
    throw new Error(errorBody.message ?? "Failed to update media");
  }

  const responseBody = (await response.json()) as { data: TastingRecord };
  return responseBody.data ?? null;
};

export const deleteTasting = async (
  id: string,
  token: string,
): Promise<void> => {
  const response = await fetch(`${config.apiBaseUrl}/tastings/${id}`, {
    method: "DELETE",
    headers: {
      Authorization: `Bearer ${token}`,
    },
  });
  if (!response.ok) {
    const errorBody = (await response.json().catch(() => ({}))) as {
      message?: string;
    };
    throw new Error(errorBody.message ?? "Failed to delete tasting");
  }
};

// Recipe API

export const fetchRecipes = async (): Promise<Recipe[]> => {
  const response = await fetch(`${config.apiBaseUrl}/recipes`);
  if (!response.ok) {
    throw new Error("Failed to fetch recipes");
  }
  const payload = (await response.json()) as { data: Recipe[] };
  return payload.data ?? [];
};

export const fetchRecipe = async (id: string): Promise<RecipeFull> => {
  const response = await fetch(`${config.apiBaseUrl}/recipes/${id}`);
  if (!response.ok) {
    throw new Error("Failed to fetch recipe");
  }
  const payload = (await response.json()) as { data: RecipeFull };
  return payload.data;
};

type UploadUrlResponse = { uploadUrl: string; key: string; publicUrl: string };

const getUploadUrl = async (
  recipeId: string,
  token: string,
  contentType: string,
  uploadType: string,
): Promise<UploadUrlResponse> => {
  const resp = await fetch(
    `${config.apiBaseUrl}/recipes/${recipeId}/upload-url`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: `Bearer ${token}`,
      },
      body: JSON.stringify({ contentType, uploadType }),
    },
  );
  if (!resp.ok) throw new Error("Failed to get upload URL");
  return resp.json() as Promise<UploadUrlResponse>;
};

export const uploadRecipeImage = async (
  recipeId: string,
  token: string,
  file: File,
): Promise<void> => {
  const { uploadUrl, key, publicUrl } = await getUploadUrl(
    recipeId,
    token,
    file.type,
    "image",
  );
  const put = await fetch(uploadUrl, {
    method: "PUT",
    body: file,
    headers: { "Content-Type": file.type },
  });
  if (!put.ok) throw new Error("Failed to upload file");
  const confirm = await fetch(
    `${config.apiBaseUrl}/recipes/${recipeId}/image`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: `Bearer ${token}`,
      },
      body: JSON.stringify({ key, publicUrl }),
    },
  );
  if (!confirm.ok) throw new Error("Failed to confirm image");
};

export const submitVoiceReview = async (
  recipeId: string,
  token: string,
  blob: Blob,
  mimeType: string,
): Promise<void> => {
  const { uploadUrl, key } = await getUploadUrl(
    recipeId,
    token,
    mimeType,
    "voice",
  );
  const put = await fetch(uploadUrl, {
    method: "PUT",
    body: blob,
    headers: { "Content-Type": mimeType },
  });
  if (!put.ok) throw new Error("Failed to upload audio");
  const confirm = await fetch(
    `${config.apiBaseUrl}/recipes/${recipeId}/voice-review`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: `Bearer ${token}`,
      },
      body: JSON.stringify({ key, mimeType }),
    },
  );
  if (!confirm.ok) throw new Error("Failed to submit review");
};

export const rerunReview = async (id: string, token: string): Promise<void> => {
  const resp = await fetch(`${config.apiBaseUrl}/recipes/reviews/${id}/rerun`, {
    method: "POST",
    headers: { Authorization: `Bearer ${token}` },
  });
  if (!resp.ok) throw new Error("Failed to rerun review");
};

export const deleteReview = async (
  id: string,
  token: string,
): Promise<void> => {
  const resp = await fetch(`${config.apiBaseUrl}/recipes/reviews/${id}`, {
    method: "DELETE",
    headers: { Authorization: `Bearer ${token}` },
  });
  if (!resp.ok) throw new Error("Failed to delete review");
};

export const deleteImage = async (id: string, token: string): Promise<void> => {
  const resp = await fetch(`${config.apiBaseUrl}/recipes/images/${id}`, {
    method: "DELETE",
    headers: { Authorization: `Bearer ${token}` },
  });
  if (!resp.ok) throw new Error("Failed to delete image");
};

export const deleteRecipe = async (
  id: string,
  token: string,
): Promise<void> => {
  const response = await fetch(`${config.apiBaseUrl}/recipes/${id}`, {
    method: "DELETE",
    headers: { Authorization: `Bearer ${token}` },
  });
  if (!response.ok) {
    const errorBody = (await response.json().catch(() => ({}))) as {
      message?: string;
    };
    throw new Error(errorBody.message ?? "Failed to delete recipe");
  }
};
