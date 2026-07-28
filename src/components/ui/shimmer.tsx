/**
 * Status label wrapper (formerly shimmer highlight).
 * Kept for API compatibility — no animated highlight; plain secondary text.
 */

import { cn } from "@/lib/utils";

export function Shimmer({
  children,
  className,
  as: Tag = "span",
}: {
  children: React.ReactNode;
  className?: string;
  as?: "span" | "p" | "div";
}) {
  return <Tag className={cn(className)}>{children}</Tag>;
}
