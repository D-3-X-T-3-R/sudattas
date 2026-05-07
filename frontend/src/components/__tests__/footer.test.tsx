import { render, screen } from "@testing-library/react";
import React from "react";
import { Footer } from "@/components/footer";

vi.mock("next/image", () => ({
  default: (props: Record<string, unknown>) => {
    const { src = "", alt = "", ...rest } = props;
    return React.createElement("img", { src: String(src), alt: String(alt), ...rest });
  },
}));

describe("Footer", () => {
  it("shows trust and policy links for launch content minimums", () => {
    render(<Footer />);

    expect(screen.getByRole("link", { name: /about sudatta's/i })).toHaveAttribute("href", "/about");
    expect(screen.getByRole("link", { name: /cancellation policy/i })).toHaveAttribute(
      "href",
      "/cancellation-policy"
    );
    expect(screen.getByRole("link", { name: /cod & prepaid guide/i })).toHaveAttribute(
      "href",
      "/payment-guide"
    );
    expect(screen.getByRole("link", { name: /size & fit guide/i })).toHaveAttribute(
      "href",
      "/size-fit-guide"
    );
  });
});
