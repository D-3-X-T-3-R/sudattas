import type { Metadata } from "next";
import { JournalPageContent } from "@/components/journal-page-content";

export const metadata: Metadata = {
  title: "Journal | Sudatta's",
  description: "The story behind Sudatta's — the people, the craft, and the journey.",
};

export default function JournalPage() {
  return <JournalPageContent />;
}
