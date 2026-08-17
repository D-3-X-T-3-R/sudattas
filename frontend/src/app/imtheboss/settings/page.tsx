"use client";

import { useQuery } from "@tanstack/react-query";
import { Ruler, Palette, Shirt, Waves, PartyPopper, Settings } from "lucide-react";
import { AdminPageShell } from "@/components/admin/admin-page-shell";
import { AdminTableCard } from "@/components/admin/admin-cards";
import { TaxonomyManagerCard } from "@/components/admin/taxonomy-manager-card";
import {
  fetchSizes,
  createSize,
  updateSize,
  deleteSize,
  fetchColors,
  createColor,
  updateColor,
  deleteColor,
  fetchFabrics,
  createFabric,
  updateFabric,
  deleteFabric,
  fetchWeaves,
  createWeave,
  updateWeave,
  deleteWeave,
  fetchOccasions,
  createOccasion,
  updateOccasion,
  deleteOccasion,
} from "@/lib/admin-queries";

export default function AdminSettingsPage() {
  const sizesQuery = useQuery({ queryKey: ["admin", "sizes"], queryFn: fetchSizes });
  const colorsQuery = useQuery({ queryKey: ["admin", "colors"], queryFn: fetchColors });
  const fabricsQuery = useQuery({ queryKey: ["admin", "fabrics"], queryFn: fetchFabrics });
  const weavesQuery = useQuery({ queryKey: ["admin", "weaves"], queryFn: fetchWeaves });
  const occasionsQuery = useQuery({ queryKey: ["admin", "occasions"], queryFn: fetchOccasions });

  return (
    <AdminPageShell
      label="Settings"
      title="Store configuration"
      description="Product taxonomy (sizes, colors, fabrics, weaves, occasions) and, eventually, payment/shipping/tax settings."
    >
      <div className="grid gap-6 lg:grid-cols-2">
        <TaxonomyManagerCard
          title="Sizes"
          icon={<Ruler className="h-4 w-4 text-[var(--color-green)]" />}
          queryKey={["admin", "sizes"]}
          items={sizesQuery.data?.map((s) => ({ id: s.sizeId, name: s.sizeName })) ?? []}
          isLoading={sizesQuery.isLoading}
          isError={sizesQuery.isError}
          noun="size"
          namePlaceholder="e.g. XL"
          onCreate={(name) => createSize(name)}
          onUpdate={(id, name) => updateSize(id, name)}
          onDelete={(id) => deleteSize(id)}
        />

        <TaxonomyManagerCard
          title="Colors"
          icon={<Palette className="h-4 w-4 text-[var(--color-green)]" />}
          queryKey={["admin", "colors"]}
          items={colorsQuery.data?.map((c) => ({ id: c.colorId, name: c.colorName })) ?? []}
          isLoading={colorsQuery.isLoading}
          isError={colorsQuery.isError}
          noun="color"
          namePlaceholder="e.g. Maroon"
          onCreate={(name) => createColor(name)}
          onUpdate={(id, name) => updateColor(id, name)}
          onDelete={(id) => deleteColor(id)}
        />

        <TaxonomyManagerCard
          title="Fabrics"
          icon={<Shirt className="h-4 w-4 text-[var(--color-green)]" />}
          queryKey={["admin", "fabrics"]}
          items={fabricsQuery.data?.map((f) => ({ id: f.fabricId, name: f.fabricName })) ?? []}
          isLoading={fabricsQuery.isLoading}
          isError={fabricsQuery.isError}
          noun="fabric"
          namePlaceholder="e.g. Kanjivaram Silk"
          onCreate={(name) => createFabric(name)}
          onUpdate={(id, name) => updateFabric(id, name)}
          onDelete={(id) => deleteFabric(id)}
        />

        <TaxonomyManagerCard
          title="Weaves"
          icon={<Waves className="h-4 w-4 text-[var(--color-green)]" />}
          queryKey={["admin", "weaves"]}
          items={weavesQuery.data?.map((w) => ({ id: w.weaveId, name: w.weaveName })) ?? []}
          isLoading={weavesQuery.isLoading}
          isError={weavesQuery.isError}
          noun="weave"
          namePlaceholder="e.g. Jamdani"
          onCreate={(name) => createWeave(name)}
          onUpdate={(id, name) => updateWeave(id, name)}
          onDelete={(id) => deleteWeave(id)}
        />

        <TaxonomyManagerCard
          title="Occasions"
          icon={<PartyPopper className="h-4 w-4 text-[var(--color-green)]" />}
          queryKey={["admin", "occasions"]}
          items={occasionsQuery.data?.map((o) => ({ id: o.occasionId, name: o.occasionName })) ?? []}
          isLoading={occasionsQuery.isLoading}
          isError={occasionsQuery.isError}
          noun="occasion"
          namePlaceholder="e.g. Wedding"
          onCreate={(name) => createOccasion(name)}
          onUpdate={(id, name) => updateOccasion(id, name)}
          onDelete={(id) => deleteOccasion(id)}
        />
      </div>

      <AdminTableCard
        title="Coming soon"
        icon={<Settings className="h-4 w-4 text-[var(--color-green)]" />}
        className="mt-6 max-w-3xl"
      >
        <div className="space-y-3 text-[15px] leading-relaxed text-[var(--color-muted)]">
          <p>Payment options, delivery areas, and tax rules will appear here once ready.</p>
        </div>
      </AdminTableCard>
    </AdminPageShell>
  );
}
