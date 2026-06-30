import { Input, type InputProps } from "@tremor/components/Input/Input";
import { cn } from "./cn";

export function BrianInput({ inputClassName, className, ...props }: InputProps) {
  return (
    <Input
      className={className}
      inputClassName={cn("brian-field", inputClassName)}
      {...props}
    />
  );
}
