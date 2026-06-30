import { SelectNative, type SelectNativeProps } from "@tremor/components/SelectNative/SelectNative";
import { cn } from "./cn";

export function BrianSelect({ className, ...props }: SelectNativeProps) {
  return <SelectNative className={cn("brian-field", className)} {...props} />;
}
