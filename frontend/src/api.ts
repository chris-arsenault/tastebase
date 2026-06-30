import { config } from "./config";
import type {
  CreateTastingInput,
  Recipe,
  RecipeFull,
  TastingRecord,
  UpdateTastingMediaInput,
} from "./types";

type TastingUploadType = "image" | "voice";

const API_UNREACHABLE_MESSAGE =
  "Could not reach the Tastebase API. Check your connection, and if you are using a VPN, private relay, or proxy, switch servers or disable it and try again.";

const API_FORBIDDEN_MESSAGE =
  "The Tastebase API blocked this request before it reached the app. VPN, private relay, or proxy IPs are commonly filtered; switch servers or disable it and try again.";

const STORAGE_FORBIDDEN_MESSAGE =
  "Storage rejected the media upload. Retry the save; if it keeps happening, sign out and back in to refresh the upload URL.";

type ErrorResponse = { message?: string };

const fetchApi = async (
  input: RequestInfo | URL,
  init?: RequestInit,
): Promise<Response> => {
  try {
    return await fetch(input, init);
  } catch (error) {
    if (error instanceof TypeError) {
      throw new Error(API_UNREACHABLE_MESSAGE);
    }
    throw error;
  }
};

const readErrorMessage = async (
  response: Response,
): Promise<string | undefined> => {
  const contentType = response.headers.get("content-type") ?? "";
  if (!contentType.includes("application/json")) return undefined;
  const body = (await response
    .json()
    .catch(() => null)) as ErrorResponse | null;
  const message = body?.message?.trim();
  return message || undefined;
};

const apiErrorMessage = async (
  response: Response,
  fallback: string,
): Promise<string> => {
  const bodyMessage = await readErrorMessage(response);
  if (bodyMessage) return bodyMessage;
  if (response.status === 403) return API_FORBIDDEN_MESSAGE;
  return fallback;
};

const assertApiOk = async (
  response: Response,
  fallback: string,
): Promise<void> => {
  if (!response.ok) {
    throw new Error(await apiErrorMessage(response, fallback));
  }
};

const assertStorageOk = (response: Response, fallback: string): void => {
  if (!response.ok) {
    throw new Error(
      response.status === 403 ? STORAGE_FORBIDDEN_MESSAGE : fallback,
    );
  }
};

const dataUrlToBlob = async (dataUrl: string): Promise<Blob> => {
  const response = await fetch(dataUrl);
  return response.blob();
};

export const uploadTastingBlob = async (
  blob: Blob,
  contentType: string,
  uploadType: TastingUploadType,
  token: string,
): Promise<string> => {
  const presign = await fetchApi(`${config.apiBaseUrl}/tastings/upload-url`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${token}`,
    },
    body: JSON.stringify({ contentType, uploadType }),
  });
  await assertApiOk(presign, "Failed to get upload URL");
  const { uploadUrl, key } = (await presign.json()) as {
    uploadUrl: string;
    key: string;
    publicUrl: string;
  };
  const put = await fetch(uploadUrl, {
    method: "PUT",
    body: blob,
    headers: { "Content-Type": contentType },
  });
  assertStorageOk(put, "Failed to upload media");
  return key;
};

export const uploadTastingMedia = async (
  dataUrl: string,
  contentType: string,
  uploadType: TastingUploadType,
  token: string,
): Promise<string> => {
  const blob = await dataUrlToBlob(dataUrl);
  return uploadTastingBlob(blob, contentType, uploadType, token);
};

export const fetchTastings = async (): Promise<TastingRecord[]> => {
  const response = await fetchApi(`${config.apiBaseUrl}/tastings`);
  await assertApiOk(response, "Failed to fetch tastings");
  const payload = (await response.json()) as { data: TastingRecord[] };
  return payload.data ?? [];
};

export const createTasting = async (
  payload: CreateTastingInput,
  token: string,
): Promise<TastingRecord | null> => {
  const response = await fetchApi(`${config.apiBaseUrl}/tastings`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${token}`,
    },
    body: JSON.stringify(payload),
  });

  await assertApiOk(response, "Failed to create tasting");

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
  const response = await fetchApi(`${config.apiBaseUrl}/tastings/${id}/rerun`, {
    method: "POST",
    headers: {
      Authorization: `Bearer ${token}`,
    },
  });
  await assertApiOk(response, "Failed to rerun pipeline");
};

export const updateTastingMedia = async (
  id: string,
  payload: UpdateTastingMediaInput,
  token: string,
): Promise<TastingRecord | null> => {
  const response = await fetchApi(`${config.apiBaseUrl}/tastings/${id}/media`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${token}`,
    },
    body: JSON.stringify(payload),
  });
  await assertApiOk(response, "Failed to update media");

  const responseBody = (await response.json()) as { data: TastingRecord };
  return responseBody.data ?? null;
};

export const deleteTasting = async (
  id: string,
  token: string,
): Promise<void> => {
  const response = await fetchApi(`${config.apiBaseUrl}/tastings/${id}`, {
    method: "DELETE",
    headers: {
      Authorization: `Bearer ${token}`,
    },
  });
  await assertApiOk(response, "Failed to delete tasting");
};

// Recipe API

export const fetchRecipes = async (): Promise<Recipe[]> => {
  const response = await fetchApi(`${config.apiBaseUrl}/recipes`);
  await assertApiOk(response, "Failed to fetch recipes");
  const payload = (await response.json()) as { data: Recipe[] };
  return payload.data ?? [];
};

export const fetchRecipe = async (id: string): Promise<RecipeFull> => {
  const response = await fetchApi(`${config.apiBaseUrl}/recipes/${id}`);
  await assertApiOk(response, "Failed to fetch recipe");
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
  const resp = await fetchApi(
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
  await assertApiOk(resp, "Failed to get upload URL");
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
  assertStorageOk(put, "Failed to upload file");
  const confirm = await fetchApi(
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
  await assertApiOk(confirm, "Failed to confirm image");
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
  assertStorageOk(put, "Failed to upload audio");
  const confirm = await fetchApi(
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
  await assertApiOk(confirm, "Failed to submit review");
};

export const rerunReview = async (id: string, token: string): Promise<void> => {
  const resp = await fetchApi(
    `${config.apiBaseUrl}/recipes/reviews/${id}/rerun`,
    {
      method: "POST",
      headers: { Authorization: `Bearer ${token}` },
    },
  );
  await assertApiOk(resp, "Failed to rerun review");
};

export const deleteReview = async (
  id: string,
  token: string,
): Promise<void> => {
  const resp = await fetchApi(`${config.apiBaseUrl}/recipes/reviews/${id}`, {
    method: "DELETE",
    headers: { Authorization: `Bearer ${token}` },
  });
  await assertApiOk(resp, "Failed to delete review");
};

export const deleteImage = async (id: string, token: string): Promise<void> => {
  const resp = await fetchApi(`${config.apiBaseUrl}/recipes/images/${id}`, {
    method: "DELETE",
    headers: { Authorization: `Bearer ${token}` },
  });
  await assertApiOk(resp, "Failed to delete image");
};

export const deleteRecipe = async (
  id: string,
  token: string,
): Promise<void> => {
  const response = await fetchApi(`${config.apiBaseUrl}/recipes/${id}`, {
    method: "DELETE",
    headers: { Authorization: `Bearer ${token}` },
  });
  await assertApiOk(response, "Failed to delete recipe");
};
