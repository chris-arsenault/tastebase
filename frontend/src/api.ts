import { config } from "./config";
import type { CreateTastingInput, Recipe, RecipeFull, TastingRecord, UpdateTastingMediaInput } from "./types";

export const fetchTastings = async (): Promise<TastingRecord[]> => {
  const response = await fetch(`${config.apiBaseUrl}/tastings`);
  if (!response.ok) {
    throw new Error("Failed to fetch tastings");
  }
  const payload = (await response.json()) as { data: TastingRecord[] };
  return payload.data ?? [];
};

export const createTasting = async (payload: CreateTastingInput, token: string): Promise<TastingRecord | null> => {
  const response = await fetch(`${config.apiBaseUrl}/tastings`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${token}`
    },
    body: JSON.stringify(payload)
  });

  if (!response.ok) {
    const errorBody = (await response.json().catch(() => ({}))) as { message?: string };
    throw new Error(errorBody.message ?? "Failed to create tasting");
  }

  if (response.status === 204) {
    return null;
  }

  const responseBody = (await response.json()) as { data: TastingRecord };
  return responseBody.data ?? null;
};

export const rerunTasting = async (id: string, token: string): Promise<void> => {
  const response = await fetch(`${config.apiBaseUrl}/tastings/${id}/rerun`, {
    method: "POST",
    headers: {
      Authorization: `Bearer ${token}`
    }
  });
  if (!response.ok) {
    const errorBody = (await response.json().catch(() => ({}))) as { message?: string };
    throw new Error(errorBody.message ?? "Failed to rerun pipeline");
  }
};

export const updateTastingMedia = async (
  id: string,
  payload: UpdateTastingMediaInput,
  token: string
): Promise<TastingRecord | null> => {
  const response = await fetch(`${config.apiBaseUrl}/tastings/${id}/media`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${token}`
    },
    body: JSON.stringify(payload)
  });

  if (!response.ok) {
    const errorBody = (await response.json().catch(() => ({}))) as { message?: string };
    throw new Error(errorBody.message ?? "Failed to update media");
  }

  const responseBody = (await response.json()) as { data: TastingRecord };
  return responseBody.data ?? null;
};

export const deleteTasting = async (id: string, token: string): Promise<void> => {
  const response = await fetch(`${config.apiBaseUrl}/tastings/${id}`, {
    method: "DELETE",
    headers: {
      Authorization: `Bearer ${token}`
    }
  });
  if (!response.ok) {
    const errorBody = (await response.json().catch(() => ({}))) as { message?: string };
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
