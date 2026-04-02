export type DatePreset = "7" | "30" | "month" | "all";

export const DATE_PRESETS: { key: DatePreset; label: string }[] = [
  { key: "7", label: "Last 7 days" },
  { key: "30", label: "Last 30 days" },
  { key: "month", label: "This month" },
  { key: "all", label: "All" },
];
