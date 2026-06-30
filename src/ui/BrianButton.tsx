import { Button, type ButtonProps } from "@tremor/components/Button/Button";
import { cn } from "./cn";

export function BrianButton({ variant = "primary", className, ...props }: ButtonProps) {
  return (
    <Button
      variant={variant === "primary" ? "secondary" : variant}
      className={cn(
        variant === "primary" &&
          "border-transparent bg-[var(--accent)] text-white hover:bg-[var(--accent)] hover:opacity-90 dark:bg-[var(--accent)] dark:text-white",
        className,
      )}
      {...props}
    />
  );
}
