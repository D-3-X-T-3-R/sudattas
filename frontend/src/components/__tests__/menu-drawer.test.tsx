import { fireEvent, render, screen } from "@testing-library/react";
import type { ReactNode } from "react";
import { MenuDrawer } from "@/components/menu-drawer";
import { consumePendingHomeMood, peekPendingHomeSection } from "@/hooks/use-scroll-to";

const mocks = vi.hoisted(() => ({ push: vi.fn(), pathname: "/bag" }));
vi.mock("next/navigation", () => ({ usePathname: () => mocks.pathname, useRouter: () => ({ push: mocks.push }) }));
vi.mock("@/components/ui/sheet", () => ({ Sheet: ({ children }: { children: ReactNode }) => <div>{children}</div> }));
vi.mock("@tanstack/react-query", () => ({
  useQuery: () => ({
    isPending: false, isError: false,
    data: { categories: [{ categoryId: "1", name: "Kurtas & Sets" }, { categoryId: "2", name: "test_hidden" }], moods: [{ moodId: "9", moodName: "Festive" }] },
  }),
}));

beforeEach(() => { sessionStorage.clear(); mocks.push.mockClear(); });

it("expands categories without navigating and links to the canonical collection", () => {
  const onClose = vi.fn();
  render(<MenuDrawer open onClose={onClose} />);
  const summary = screen.getByRole("button", { name: "Categories" });
  expect(summary).toHaveAttribute("aria-expanded", "false");
  fireEvent.click(summary);
  expect(summary).toHaveAttribute("aria-expanded", "true");
  expect(onClose).not.toHaveBeenCalled();
  const link = screen.getByRole("link", { name: "Kurtas & Sets" });
  expect(link).toHaveAttribute("href", "/collections/kurtas-sets");
  expect(screen.queryByText("test_hidden")).not.toBeInTheDocument();
  fireEvent.click(summary);
  expect(summary).toHaveAttribute("aria-expanded", "false");
});

it("queues a selected mood and returns to the homepage product section", () => {
  const onClose = vi.fn();
  render(<MenuDrawer open onClose={onClose} />);
  fireEvent.click(screen.getByText("Moods"));
  fireEvent.click(screen.getByRole("button", { name: "Festive" }));
  expect(consumePendingHomeMood()).toBe("9");
  expect(consumePendingHomeMood()).toBeNull();
  expect(peekPendingHomeSection()).toEqual({ id: "shop", fromOtherPage: true });
  expect(mocks.push).toHaveBeenCalledWith("/");
  expect(onClose).toHaveBeenCalledOnce();
});
