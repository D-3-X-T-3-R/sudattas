import { useState } from "react";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { Sheet } from "@/components/ui/sheet";

function SheetHarness() {
  const [open, setOpen] = useState(false);
  return (
    <div>
      <button type="button" onClick={() => setOpen(true)}>
        Open Sheet
      </button>
      <button type="button">Outside Button</button>
      <Sheet open={open} onClose={() => setOpen(false)} title="Test Sheet">
        <button type="button">Primary Action</button>
        <button type="button">Secondary Action</button>
      </Sheet>
    </div>
  );
}

describe("Sheet focus behavior", () => {
  it("traps tab focus while open and returns focus to trigger on close", async () => {
    const user = userEvent.setup();
    render(<SheetHarness />);

    const trigger = screen.getByRole("button", { name: "Open Sheet" });
    trigger.focus();
    await user.click(trigger);

    const dialog = await screen.findByRole("dialog");
    const closeButton = screen.getByRole("button", { name: "Close" });
    const primary = screen.getByRole("button", { name: "Primary Action" });
    const secondary = screen.getByRole("button", { name: "Secondary Action" });

    await waitFor(() => {
      expect(document.activeElement).toBe(closeButton);
    });

    await user.tab();
    expect(document.activeElement).toBe(primary);
    expect(dialog.contains(document.activeElement)).toBe(true);

    await user.tab();
    expect(document.activeElement).toBe(secondary);
    expect(dialog.contains(document.activeElement)).toBe(true);

    await user.tab();
    expect(document.activeElement).toBe(closeButton);
    expect(dialog.contains(document.activeElement)).toBe(true);

    await user.keyboard("{Escape}");

    await waitFor(() => {
      expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
    });
    expect(document.activeElement).toBe(trigger);
  });
});
