import { forwardAdminGraphql } from "@/lib/admin-graphql-server";

const PRODUCT_ROOTS = [
  "searchCategory",
  "createCategory",
  "searchProduct",
  "createProduct",
  "updateProduct",
  "deleteProduct",
  "searchProductImage",
  "deleteProductImage",
  "searchSize",
  "searchColor",
  "searchFabric",
  "searchWeave",
  "searchOccasion",
  "searchProductMood",
  "createProductMood",
  "searchProductMoodMapping",
  "createProductMoodMapping",
  "deleteProductMoodMapping",
  "createProductVariant",
  "updateProductVariant",
  "deleteProductVariant",
  "createInventoryItem",
  "searchInventoryItem",
  "updateInventoryItem",
  "getPresignedUploadUrl",
  "confirmImageUpload",
  "syncProductImages",
  "shopHighlightMoods",
];

export async function POST(request: Request) {
  return forwardAdminGraphql(request, { allowedRoots: PRODUCT_ROOTS });
}

