"use client";

import * as React from "react";
import * as DialogPrimitive from "@radix-ui/react-dialog";
import { X } from "lucide-react";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";

const Dialog = DialogPrimitive.Root;
const DialogTrigger = DialogPrimitive.Trigger;
const DialogClose = DialogPrimitive.Close;
const DialogPortal = DialogPrimitive.Portal;

const DialogOverlay = React.forwardRef<
  React.ElementRef<typeof DialogPrimitive.Overlay>,
  React.ComponentPropsWithoutRef<typeof DialogPrimitive.Overlay>
>(({ className, ...props }, ref) => (
  <DialogPrimitive.Overlay
    ref={ref}
    className={cn(
      "fixed inset-0 z-40 bg-black/30 backdrop-blur-[1px]",
      className
    )}
    {...props}
  />
));
DialogOverlay.displayName = DialogPrimitive.Overlay.displayName;

const DialogContent = React.forwardRef<
  React.ElementRef<typeof DialogPrimitive.Content>,
  React.ComponentPropsWithoutRef<typeof DialogPrimitive.Content> & {
    title?: string;
    /** Merged with default title styles when `title` is set. */
    titleClassName?: string;
    showClose?: boolean;
    contentClassName?: string;
  }
>(({ className, children, title, titleClassName, showClose = true, contentClassName, ...props }, ref) => (
  <DialogPortal>
    <DialogOverlay />
    <DialogPrimitive.Content
      ref={ref}
      aria-describedby={undefined}
      className={cn(
        "fixed left-1/2 top-1/2 z-50 w-[92vw] max-w-4xl -translate-x-1/2 -translate-y-1/2 overflow-hidden rounded-lg border border-[var(--color-line)] bg-[var(--color-surface)] shadow-[var(--shadow-soft)] focus:outline-none",
        className
      )}
      {...props}
    >
      {(title || showClose) ? (
        <div className="flex items-center justify-between border-b border-[var(--color-line)] px-4 py-3">
          {title ? (
            <DialogPrimitive.Title
              className={cn(
                "text-[11px] font-semibold uppercase tracking-[0.2em] text-[var(--color-muted)]",
                titleClassName
              )}
            >
              {title}
            </DialogPrimitive.Title>
          ) : (
            <DialogPrimitive.Title className="sr-only">Dialog</DialogPrimitive.Title>
          )}
          {showClose && (
            <DialogPrimitive.Close asChild>
              <Button variant="outline" size="icon" aria-label="Close">
                <X className="h-5 w-5" />
              </Button>
            </DialogPrimitive.Close>
          )}
        </div>
      ) : (
        <DialogPrimitive.Title className="sr-only">Dialog</DialogPrimitive.Title>
      )}
      <div className={cn("p-4 sm:p-5", contentClassName)}>{children}</div>
    </DialogPrimitive.Content>
  </DialogPortal>
));
DialogContent.displayName = DialogPrimitive.Content.displayName;

export {
  Dialog,
  DialogPortal,
  DialogOverlay,
  DialogTrigger,
  DialogClose,
  DialogContent,
};
