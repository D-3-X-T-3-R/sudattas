const notFoundMock = vi.fn(() => {
  throw new Error("NEXT_NOT_FOUND");
});
const loadCollectionByCategorySlugMock = vi.fn();

vi.mock("next/navigation", () => ({
  notFound: () => notFoundMock(),
}));

vi.mock("@/lib/storefront-collection-page", () => ({
  loadCollectionByCategorySlug: (...args: unknown[]) => loadCollectionByCategorySlugMock(...args),
}));

import CollectionSlugPage from "@/app/collections/[slug]/page";

describe("Collection slug page", () => {
  beforeEach(() => {
    notFoundMock.mockClear();
    loadCollectionByCategorySlugMock.mockReset();
  });

  it("uses notFound for unknown slugs", async () => {
    loadCollectionByCategorySlugMock.mockResolvedValue(null);

    await expect(
      CollectionSlugPage({ params: Promise.resolve({ slug: "unknown-collection" }) })
    ).rejects.toThrow("NEXT_NOT_FOUND");
    expect(notFoundMock).toHaveBeenCalled();
  });
});
