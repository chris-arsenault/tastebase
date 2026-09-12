import { assertApiOk, fetchApi } from "./api";
import { config } from "./config";
import type {
  BookRecommendation,
  PublicationRecommendation,
  ShelfItem,
  ShelfStatus,
} from "./types";

async function fetchList<T>(path: string): Promise<T[]> {
  const response = await fetchApi(`${config.apiBaseUrl}${path}`);
  await assertApiOk(response, "Failed to fetch the bookshelf");
  const payload = (await response.json()) as { data: T[] };
  return payload.data ?? [];
}

export async function fetchShelf(): Promise<ShelfItem[]> {
  const [books, publications] = await Promise.all([
    fetchList<BookRecommendation>("/books/public"),
    fetchList<PublicationRecommendation>("/books/publications"),
  ]);
  return [
    ...books.map((book) => ({ ...book, kind: "book" as const })),
    ...publications.map((publication) => ({
      ...publication,
      kind: "publication" as const,
    })),
  ];
}

async function updateItem(
  item: ShelfItem,
  action: string,
  body: object,
  token: string,
): Promise<ShelfItem> {
  const base = item.kind === "book" ? "/books" : "/books/publications";
  const response = await fetchApi(
    `${config.apiBaseUrl}${base}/${item.id}/${action}`,
    {
      method: "PUT",
      headers: {
        "Content-Type": "application/json",
        Authorization: `Bearer ${token}`,
      },
      body: JSON.stringify(body),
    },
  );
  await assertApiOk(response, "Failed to update bookshelf item");
  const payload = (await response.json()) as {
    data: BookRecommendation | PublicationRecommendation;
  };
  return { ...payload.data, kind: item.kind } as ShelfItem;
}

export const updateShelfStatus = (
  item: ShelfItem,
  status: ShelfStatus,
  token: string,
) => updateItem(item, "status", { status }, token);

export const saveShelfReview = (
  item: ShelfItem,
  rating: number,
  writeup: string,
  token: string,
) => updateItem(item, "review", { rating, writeup }, token);
