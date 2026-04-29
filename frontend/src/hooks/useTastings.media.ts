import { uploadTastingMedia } from "../api";
import type { CreateTastingInput, UpdateTastingMediaInput } from "../types";
import type { FormState } from "./useTastings";

export type MediaData = {
  imageBase64: string;
  imageMimeType: string;
  ingredientsImageBase64: string;
  ingredientsImageMimeType: string;
  nutritionImageBase64: string;
  nutritionImageMimeType: string;
  audioBase64: string;
  audioMimeType: string;
};

export type UploadedKeys = {
  imageKey?: string;
  imageMimeType?: string;
  ingredientsImageKey?: string;
  ingredientsImageMimeType?: string;
  nutritionImageKey?: string;
  nutritionImageMimeType?: string;
  voiceKey?: string;
  voiceMimeType?: string;
};

type Slot = {
  dataUrl: string;
  mimeType: string;
  uploadType: "image" | "voice";
  fallbackMime: string;
};

const uploadSlot = async (
  slot: Slot,
  token: string,
): Promise<{ key?: string; mimeType?: string }> => {
  if (!slot.dataUrl) return {};
  const mime = slot.mimeType || slot.fallbackMime;
  const key = await uploadTastingMedia(
    slot.dataUrl,
    mime,
    slot.uploadType,
    token,
  );
  return { key, mimeType: mime };
};

export const uploadAllMedia = async (
  mediaData: MediaData,
  token: string,
): Promise<UploadedKeys> => {
  const [image, ingredients, nutrition, voice] = await Promise.all([
    uploadSlot(
      {
        dataUrl: mediaData.imageBase64,
        mimeType: mediaData.imageMimeType,
        uploadType: "image",
        fallbackMime: "image/jpeg",
      },
      token,
    ),
    uploadSlot(
      {
        dataUrl: mediaData.ingredientsImageBase64,
        mimeType: mediaData.ingredientsImageMimeType,
        uploadType: "image",
        fallbackMime: "image/jpeg",
      },
      token,
    ),
    uploadSlot(
      {
        dataUrl: mediaData.nutritionImageBase64,
        mimeType: mediaData.nutritionImageMimeType,
        uploadType: "image",
        fallbackMime: "image/jpeg",
      },
      token,
    ),
    uploadSlot(
      {
        dataUrl: mediaData.audioBase64,
        mimeType: mediaData.audioMimeType,
        uploadType: "voice",
        fallbackMime: "audio/webm",
      },
      token,
    ),
  ]);
  return {
    imageKey: image.key,
    imageMimeType: image.mimeType,
    ingredientsImageKey: ingredients.key,
    ingredientsImageMimeType: ingredients.mimeType,
    nutritionImageKey: nutrition.key,
    nutritionImageMimeType: nutrition.mimeType,
    voiceKey: voice.key,
    voiceMimeType: voice.mimeType,
  };
};

export const buildEditMediaPayload = (
  keys: UploadedKeys,
): UpdateTastingMediaInput | null => {
  if (!keys.imageKey && !keys.ingredientsImageKey && !keys.nutritionImageKey) {
    return null;
  }
  return {
    imageKey: keys.imageKey,
    ingredientsImageKey: keys.ingredientsImageKey,
    nutritionImageKey: keys.nutritionImageKey,
  };
};

const trimOrUndefined = (value: string) => value.trim() || undefined;

const toNumberOrNull = (value: string) => {
  if (!value.trim()) return null;
  const parsed = Number(value);
  return Number.isNaN(parsed) ? null : parsed;
};

export const buildCreatePayload = (
  formData: FormState,
  keys: UploadedKeys,
): CreateTastingInput => ({
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
  imageKey: keys.imageKey,
  imageMimeType: keys.imageMimeType,
  ingredientsImageKey: keys.ingredientsImageKey,
  ingredientsImageMimeType: keys.ingredientsImageMimeType,
  nutritionImageKey: keys.nutritionImageKey,
  nutritionImageMimeType: keys.nutritionImageMimeType,
  voiceKey: keys.voiceKey,
  voiceMimeType: keys.voiceMimeType,
});
