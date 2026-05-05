import { render, screen } from "@testing-library/react";
import NotFound from "@/app/not-found";

describe("app not-found page", () => {
  it("renders branded recovery CTAs", () => {
    render(<NotFound />);

    expect(screen.getByText(/page not found/i)).toBeInTheDocument();
    expect(screen.getByRole("link", { name: /continue shopping/i })).toHaveAttribute("href", "/");
    expect(screen.getByRole("link", { name: /contact support/i })).toHaveAttribute(
      "href",
      "/contact-support"
    );
  });
});
