import { Badge, type BadgeProps } from "@tremor/components/Badge/Badge";
import { cn } from "./cn";

export function BrianBadge({ className, variant = "neutral", ...props }: BadgeProps) {
  return <Badge variant={variant} className={cn(className)} {...props} />;
}
