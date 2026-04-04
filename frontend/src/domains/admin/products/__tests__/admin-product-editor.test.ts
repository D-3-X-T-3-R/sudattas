import { adminProductFormSchema } from "@/lib/schemas";

describe("Admin product editor schema", () => {
  it("accepts valid product form input", () => {
    const parsed = adminProductFormSchema.safeParse({
      name: "Test Product",
      description: "Good product",
      priceRupees: "1299.00",
      categoryId: "1",
      sku: "SKU_123",
      slug: "test-product",
      fabric: "Cotton",
      weave: "Handloom",
      occasion: "Festive",
      hasBlousePiece: true,
      careInstructions: "Dry clean",
      productStatusId: "2",
    });

    expect(parsed.success).toBe(true);
  });

  it("rejects invalid price and invalid slug", () => {
    const parsed = adminProductFormSchema.safeParse({
      name: "Test Product",
      description: "Good product",
      priceRupees: "0",
      categoryId: "1",
      sku: "SKU_123",
      slug: "bad slug with spaces",
      fabric: "Cotton",
      weave: "Handloom",
      occasion: "Festive",
      hasBlousePiece: true,
      careInstructions: "Dry clean",
      productStatusId: "2",
    });

    expect(parsed.success).toBe(false);
  });
});
