import type { ComponentPropsWithoutRef } from "react";
import { Label } from "@tremor/components/Label/Label";
import { cn } from "./cn";

export function BrianLabel({ className, ...props }: ComponentPropsWithoutRef<typeof Label>) {
  return <Label className={cn("text-[var(--text-secondary)]", className)} {...props} />;
}
