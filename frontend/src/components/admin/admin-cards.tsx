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
    <Card className={cn("rounded-lg border-[var(--color-line)] bg-[var(--admin-surface-muted)] shadow-[var(--admin-card-shadow)]", className)}>
      <CardTitle className="flex items-center gap-2 text-[var(--color-muted)]">
        {icon}
        {title}
      </CardTitle>
      <CardContent className={cn("mt-3", contentClassName)}>{children}</CardContent>
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
