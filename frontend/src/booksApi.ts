import { assertApiOk, fetchApi } from "./api";
import { config } from "./config";
import type { BookRecommendation, BookStatus } from "./types";

const readBookResponse = async (
  response: Response,
): Promise<BookRecommendation> => {
  const payload = (await response.json()) as { data: BookRecommendation };
  return payload.data;
};

export const fetchBooks = async (
  token: string,
): Promise<BookRecommendation[]> => {
  const response = await fetchApi(`${config.apiBaseUrl}/books`, {
    headers: { Authorization: `Bearer ${token}` },
  });
  await assertApiOk(response, "Failed to fetch your book recommendations");
  const payload = (await response.json()) as { data: BookRecommendation[] };
  return payload.data ?? [];
};

export const fetchPublicBooks = async (): Promise<BookRecommendation[]> => {
  const response = await fetchApi(`${config.apiBaseUrl}/books/public`);
  await assertApiOk(response, "Failed to fetch the public bookshelf");
  const payload = (await response.json()) as { data: BookRecommendation[] };
  return payload.data ?? [];
};

export const updateBookStatus = async (
  id: string,
  status: BookStatus,
  token: string,
): Promise<BookRecommendation> => {
  const response = await fetchApi(`${config.apiBaseUrl}/books/${id}/status`, {
    method: "PUT",
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${token}`,
    },
    body: JSON.stringify({ status }),
  });
  await assertApiOk(response, "Failed to update reading status");
  return readBookResponse(response);
};

export const saveBookReview = async (
  id: string,
  rating: number,
  writeup: string,
  token: string,
): Promise<BookRecommendation> => {
  const response = await fetchApi(`${config.apiBaseUrl}/books/${id}/review`, {
    method: "PUT",
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${token}`,
    },
    body: JSON.stringify({ rating, writeup }),
  });
  await assertApiOk(response, "Failed to save your book review");
  return readBookResponse(response);
};

export const updateBookVisibility = async (
  id: string,
  isPublic: boolean,
  token: string,
): Promise<BookRecommendation> => {
  const response = await fetchApi(
    `${config.apiBaseUrl}/books/${id}/visibility`,
    {
      method: "PUT",
      headers: {
        "Content-Type": "application/json",
        Authorization: `Bearer ${token}`,
      },
      body: JSON.stringify({ isPublic }),
    },
  );
  await assertApiOk(response, "Failed to update book visibility");
  return readBookResponse(response);
};
