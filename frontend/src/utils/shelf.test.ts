import { describe, expect, it } from "vitest";
import { renderToStaticMarkup } from "react-dom/server";
import { createElement } from "react";
import type { ShelfItem } from "../types";
import {
  collectTagFacets,
  filterShelf,
  isReviewed,
  type ShelfFilters,
} from "./shelf";
import { BookCard } from "../components/BookCard";

const common = {
  id: "example",
  title: "Example",
  summary: "Summary",
  whyRecommended: "Reason",
  tags: [
    { key: "fit", value: "high" },
    { key: "outlook", value: "humanist" },
  ],
  rating: null,
  writeup: "",
  recommendedAt: "2026-09-12T00:00:00Z",
  createdAt: "2026-09-12T00:00:00Z",
  updatedAt: "2026-09-12T00:00:00Z",
};
const book: ShelfItem = {
  ...common,
  kind: "book",
  author: "Author",
  status: "read",
  pageCount: 120,
  purchaseLink: null,
  readAt: null,
  isPublic: true,
};
const publication: ShelfItem = {
  ...common,
  id: "publication",
  kind: "publication",
  publisher: "Publisher",
  status: "cancelled",
  rating: 4,
  writeup: "Worth reading",
  homepage: "https://example.org",
  publicationType: "journal",
  cadence: { label: "monthly", issuesPerYear: 12 },
  audienceLevel: "academic-adjacent",
  editorialHome: { city: "London", country: "United Kingdom" },
  outlook: { summary: "A valuable challenge", relationship: "contrast" },
  subscriptions: [
    {
      label: "Annual print",
      formats: ["print"],
      region: "UK",
      accessModel: "subscription",
      price: {
        amount: "49.95",
        currency: "GBP",
        period: "year",
        priceType: "introductory",
      },
      url: "https://example.org/subscribe",
      verifiedAt: "2026-09-12",
      notes: "Delivery included",
    },
  ],
};
const all: ShelfFilters = {
  status: "all",
  kind: "all",
  reviewedOnly: false,
  tags: {},
};
const noop = () => {};

describe("public bookshelf", () => {
  it("shows unreviewed books and reviewed publications by default", () => {
    expect(filterShelf([book, publication], all)).toEqual([book, publication]);
  });

  it("defines reviewed by feedback, independent of reading and subscription status", () => {
    expect(isReviewed(book)).toBe(false);
    expect(isReviewed({ ...book, rating: 4, writeup: "   " })).toBe(false);
    expect(isReviewed({ ...book, writeup: "No rating" })).toBe(false);
    const unfinished: ShelfItem = {
      ...book,
      status: "did_not_finish",
      rating: 2,
      writeup: "Stopped halfway",
    };
    expect(
      filterShelf([book, unfinished, publication], {
        ...all,
        reviewedOnly: true,
      }),
    ).toEqual([unfinished, publication]);
  });

  it("combines review, type, status and reusable outlook tags without confusing fit and alignment", () => {
    expect(
      filterShelf([book, publication], {
        kind: "publication",
        status: "cancelled",
        reviewedOnly: true,
        tags: { fit: ["high"], outlook: ["humanist"] },
      }),
    ).toEqual([publication]);
    expect(
      filterShelf([book, publication], {
        ...all,
        tags: { outlook: ["rationalist"] },
      }),
    ).toEqual([]);
  });

  it("ORs values within a key, ANDs across keys, and searches title/creator/summary", () => {
    const other: ShelfItem = {
      ...book,
      id: "other",
      title: "Other",
      tags: [{ key: "outlook", value: "rationalist" }],
    };
    expect(
      filterShelf([book, other, publication], {
        ...all,
        tags: { outlook: ["humanist", "rationalist"] },
      }),
    ).toEqual([book, other, publication]);
    expect(
      filterShelf([book, other, publication], {
        ...all,
        tags: { outlook: ["humanist", "rationalist"], fit: ["low"] },
      }),
    ).toEqual([]);
    expect(
      filterShelf([book, other, publication], { ...all, search: "publisher" }),
    ).toEqual([publication]);
  });
});

describe("bookshelf facets", () => {
  it("counts facet values against the other active filters", () => {
    const other: ShelfItem = {
      ...book,
      id: "other",
      title: "Other",
      tags: [{ key: "outlook", value: "rationalist" }],
    };
    const facets = collectTagFacets([book, other, publication], {
      ...all,
      kind: "book",
      tags: { outlook: ["humanist"] },
    });
    const outlook = facets.find((facet) => facet.key === "outlook");
    // Selecting a value in the same key must not shrink its sibling counts.
    expect(outlook?.values).toEqual([
      { value: "humanist", count: 1 },
      { value: "rationalist", count: 1 },
    ]);
    const fit = facets.find((facet) => facet.key === "fit");
    // The outlook filter and kind=book both apply to the fit facet.
    expect(fit?.values).toEqual([{ value: "high", count: 1 }]);
  });
});

describe("bookshelf cards", () => {
  it("renders publication semantics publicly with no owner controls", () => {
    const html = renderToStaticMarkup(
      createElement(BookCard, {
        book: publication,
        editable: false,
        saving: false,
        onStatus: noop,
        onReview: noop,
      }),
    );
    for (const value of [
      "49.95",
      "GBP",
      "year",
      "Introductory",
      "Verified 2026-09-12",
      "London, United Kingdom",
      "Academic adjacent",
      "Contrast",
      "A valuable challenge",
      "Worth reading",
    ]) {
      expect(html).toContain(value);
    }
    expect(html).not.toContain("Subscription status");
    expect(html).not.toContain("Save review");
    expect(html).not.toContain("Share this review");
  });

  it("shows unreviewed state and the correct owner controls for each kind", () => {
    const render = (item: ShelfItem) =>
      renderToStaticMarkup(
        createElement(BookCard, {
          book: item,
          editable: true,
          saving: false,
          onStatus: noop,
          onReview: noop,
        }),
      );
    expect(render(book)).toContain("Not yet reviewed");
    expect(render(book)).toContain("Reading status");
    expect(render(publication)).toContain("Subscription status");
    expect(render(publication)).not.toContain("Reading status");
  });
});
