import { beforeEach, describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({
  forwardAdminGraphql: vi.fn(),
}));

vi.mock("@/lib/admin-graphql-server", () => ({
  forwardAdminGraphql: mocks.forwardAdminGraphql,
}));

import { POST } from "@/app/api/admin/orders/route";

describe("POST /api/admin/orders", () => {
  beforeEach(() => {
    mocks.forwardAdminGraphql.mockReset();
  });

  it("forwards through admin proxy with pickup-target mutation allowlisted", async () => {
    mocks.forwardAdminGraphql.mockResolvedValue(
      Response.json({ ok: true, data: { updatePickupTarget: { orderId: "67" } } })
    );

    const request = new Request("http://localhost/api/admin/orders", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ query: "mutation { updatePickupTarget(...) { orderId } }" }),
    });

    const response = await POST(request);
    const body = (await response.json()) as { ok: boolean };

    expect(body.ok).toBe(true);
    expect(mocks.forwardAdminGraphql).toHaveBeenCalledTimes(1);
    const [, options] = mocks.forwardAdminGraphql.mock.calls[0] as [
      Request,
      { allowedRoots: string[] }
    ];
    expect(options.allowedRoots).toContain("updatePickupTarget");
    expect(options.allowedRoots).toEqual(
      expect.arrayContaining(["searchOrder", "searchOrderStatus", "updateOrder", "createOrder"])
    );
  });
});
