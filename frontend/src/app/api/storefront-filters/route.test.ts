import { expect, it, vi } from "vitest";
const mocks = vi.hoisted(() => ({ products: vi.fn(), highlights: vi.fn() }));
vi.mock("@/lib/forwarded-ip", () => ({ forwardedIpHeadersFromRequest: () => ({}) }));
vi.mock("@/lib/server-guest-session", () => ({
  mintGuestSessionIdSingleFlight: async () => "guest",
  withRecoveredGuestSession: async (id: string, _headers: unknown, load: (id: string) => Promise<unknown>) => ({ value: await load(id) }),
}));
vi.mock("@/lib/storefront-queries", () => ({
  fetchCategoriesWithSession: async () => [{ categoryId: "1", name: "Sarees" }, { categoryId: "2", name: "Empty" }],
  fetchOccasionsWithSession: async () => [],
  fetchProductMoodsWithSession: async () => [{ moodId: "3", moodName: "Festive" }, { moodId: "4", moodName: "Empty mood" }],
  fetchShopHighlightMoodsWithSession: mocks.highlights,
  fetchProductsListWithSession: mocks.products,
}));
import { GET } from "./route";

it("only returns groups with listed products, including products without stock or thumbnails", async () => {
  mocks.products.mockImplementation(async (_id, filter) =>
    filter.categoryId === "1" || filter.moodId === "3"
      ? [{ productId: "99", stockQuantity: "0", images: [] }]
      : []
  );
  const response = await GET(new Request("http://localhost/api/storefront-filters?listedOnly=1"));
  const data = await response.json();
  expect(data.error).toBeNull();
  expect(data.categories).toEqual([{ categoryId: "1", name: "Sarees" }]);
  expect(data.moods).toEqual([{ moodId: "3", moodName: "Festive" }]);
  expect(mocks.products).toHaveBeenCalledTimes(4);
  expect(mocks.highlights).not.toHaveBeenCalled();
});
