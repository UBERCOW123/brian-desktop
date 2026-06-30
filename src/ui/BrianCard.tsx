import type { ReactNode } from "react";
import { Card, type CardProps } from "@tremor/components/Card/Card";
import { cn } from "./cn";

export type BrianGlassDepth = "surface" | "elevated" | "solid";

const depthClass: Record<BrianGlassDepth, string> = {
  solid: "brian-surface",
  surface: "brian-glass brian-glass--surface",
  elevated: "brian-glass brian-glass--elevated",
};

export type BrianCardProps = CardProps & {
  glass?: BrianGlassDepth;
  children: ReactNode;
};

export function BrianCard({ glass = "elevated", className, children, ...props }: BrianCardProps) {
  return (
    <Card className={cn(depthClass[glass], "p-0 shadow-none", className)} {...props}>
      {children}
    </Card>
  );
}
