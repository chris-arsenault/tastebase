import { uploadTastingBlob, uploadTastingMedia } from "../api";
import type { CreateTastingInput, UpdateTastingMediaInput } from "../types";
import type { FormState } from "./useTastings";

export type MediaData = {
  imageBase64: string;
  imageMimeType: string;
  ingredientsImageBase64: string;
  ingredientsImageMimeType: string;
  nutritionImageBase64: string;
  nutritionImageMimeType: string;
  audioBlob: Blob | null;
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

type ImageSlot = {
  dataUrl: string;
  mimeType: string;
  fallbackMime: string;
};

const uploadImageSlot = async (
  slot: ImageSlot,
  token: string,
): Promise<{ key?: string; mimeType?: string }> => {
  if (!slot.dataUrl) return {};
  const mime = slot.mimeType || slot.fallbackMime;
  const key = await uploadTastingMedia(slot.dataUrl, mime, "image", token);
  return { key, mimeType: mime };
};

const uploadVoiceBlob = async (
  blob: Blob | null,
  mimeType: string,
  token: string,
): Promise<{ key?: string; mimeType?: string }> => {
  if (!blob) return {};
  const mime = mimeType || blob.type || "audio/webm";
  const key = await uploadTastingBlob(blob, mime, "voice", token);
  return { key, mimeType: mime };
};

export const uploadAllMedia = async (
  mediaData: MediaData,
  token: string,
): Promise<UploadedKeys> => {
  const [image, ingredients, nutrition, voice] = await Promise.all([
    uploadImageSlot(
      {
        dataUrl: mediaData.imageBase64,
        mimeType: mediaData.imageMimeType,
        fallbackMime: "image/jpeg",
      },
      token,
    ),
    uploadImageSlot(
      {
        dataUrl: mediaData.ingredientsImageBase64,
        mimeType: mediaData.ingredientsImageMimeType,
        fallbackMime: "image/jpeg",
      },
      token,
    ),
    uploadImageSlot(
      {
        dataUrl: mediaData.nutritionImageBase64,
        mimeType: mediaData.nutritionImageMimeType,
        fallbackMime: "image/jpeg",
      },
      token,
    ),
    uploadVoiceBlob(mediaData.audioBlob, mediaData.audioMimeType, token),
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
