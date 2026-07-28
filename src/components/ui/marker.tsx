/**
 * Conversation marker — status, note, bordered row, or labeled separator.
 * API aligned with shadcn Marker (base-rhea docs).
 */

import * as React from "react";
import { Slot } from "@radix-ui/react-slot";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "@/lib/utils";

const markerVariants = cva(
  "group/marker flex w-full min-w-0 items-start gap-2 text-[13px] leading-snug text-[var(--text-secondary)]",
  {
    variants: {
      variant: {
        default: "py-1",
        border:
          "py-2 border-b border-[var(--border-subtle)] last:border-b-0",
        separator:
          "my-3 items-center justify-center gap-3 py-0 text-[12px] text-[var(--text-tertiary)] before:h-px before:flex-1 before:bg-[var(--border-subtle)] after:h-px after:flex-1 after:bg-[var(--border-subtle)]",
      },
    },
    defaultVariants: {
      variant: "default",
    },
  },
);

export interface MarkerProps
  extends React.HTMLAttributes<HTMLDivElement>,
    VariantProps<typeof markerVariants> {
  /** Render as a different element (e.g. button / a). */
  asChild?: boolean;
}

export function Marker({
  className,
  variant,
  asChild = false,
  ...props
}: MarkerProps) {
  const Comp = asChild ? Slot : "div";
  return (
    <Comp
      data-slot="marker"
      data-variant={variant ?? "default"}
      className={cn(markerVariants({ variant }), className)}
      {...props}
    />
  );
}

export function MarkerIcon({
  className,
  ...props
}: React.HTMLAttributes<HTMLSpanElement>) {
  return (
    <span
      data-slot="marker-icon"
      aria-hidden
      className={cn(
        "mt-0.5 inline-flex size-4 shrink-0 items-center justify-center text-[var(--text-tertiary)] [&_svg]:size-3.5",
        className,
      )}
      {...props}
    />
  );
}

export function MarkerContent({
  className,
  ...props
}: React.HTMLAttributes<HTMLSpanElement>) {
  return (
    <span
      data-slot="marker-content"
      className={cn("min-w-0 flex-1 text-pretty", className)}
      {...props}
    />
  );
}

export { markerVariants };
