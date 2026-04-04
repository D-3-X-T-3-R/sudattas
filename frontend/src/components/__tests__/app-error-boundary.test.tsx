import { render, screen } from "@testing-library/react";
import { AppErrorBoundary } from "@/components/app-error-boundary";

const trackClientTelemetryMock = vi.fn();

vi.mock("@/lib/client-telemetry", () => ({
  trackClientTelemetry: (...args: unknown[]) => trackClientTelemetryMock(...args),
}));

function Crasher() {
  throw new Error("boom");
}

describe("AppErrorBoundary", () => {
  beforeEach(() => {
    trackClientTelemetryMock.mockReset();
  });

  it("renders fallback UI and emits telemetry when child throws", () => {
    render(
      <AppErrorBoundary>
        <Crasher />
      </AppErrorBoundary>
    );

    expect(screen.getByText("Something came undone.")).toBeInTheDocument();
    expect(trackClientTelemetryMock).toHaveBeenCalledTimes(1);
  });
});
