"use client";

import type { ReactNode } from "react";
import { Card, CardContent, CardTitle } from "@/components/ui/card";
import { cn } from "@/lib/utils";

function AdminCardBase({
  title,
  icon,
  children,
  className,
  contentClassName,
}: {
  title: string;
  icon?: ReactNode;
  children: ReactNode;
  className?: string;
  contentClassName?: string;
}) {
  return (
    <Card className={cn("rounded-2xl border-[var(--color-line)] bg-[var(--admin-surface-muted)] p-5 shadow-[var(--admin-card-shadow)] md:p-6", className)}>
      <CardTitle className="flex items-center gap-2.5 text-sm font-semibold normal-case tracking-normal text-[var(--color-ink)] md:text-[15px]">
        {icon}
        {title}
      </CardTitle>
      <CardContent className={cn("mt-4", contentClassName)}>{children}</CardContent>
    </Card>
  );
}

export function AdminFilterCard(props: {
  title?: string;
  icon?: ReactNode;
  children: ReactNode;
  className?: string;
  contentClassName?: string;
}) {
  return (
    <AdminCardBase
      title={props.title ?? "Filters"}
      icon={props.icon}
      className={props.className}
      contentClassName={props.contentClassName}
    >
      {props.children}
    </AdminCardBase>
  );
}

export function AdminTableCard(props: {
  title: string;
  icon?: ReactNode;
  children: ReactNode;
  className?: string;
  contentClassName?: string;
}) {
  return (
    <AdminCardBase
      title={props.title}
      icon={props.icon}
      className={props.className}
      contentClassName={props.contentClassName}
    >
      {props.children}
    </AdminCardBase>
  );
}
