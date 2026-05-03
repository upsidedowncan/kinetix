<script lang="ts">
  import { cn } from "$lib/utils";

  let className: string = "";
  export { className as class };
  export let variant: "default" | "primary" | "secondary" | "ghost" | "outline" = "default";
  export let size: "sm" | "md" | "lg" = "md";
  export let disabled: boolean = false;

  const variants = {
    default: "bg-surfaceVariant text-white hover:bg-surfaceVariant/80",
    primary: "bg-primary text-primary-foreground hover:bg-primary/90",
    secondary: "bg-secondary text-secondary-foreground hover:bg-secondary/90",
    ghost: "hover:bg-surfaceVariant/50 text-white",
    outline: "border border-surfaceVariant text-white hover:bg-surfaceVariant/50"
  };

  const sizes = {
    sm: "h-8 px-3 text-xs",
    md: "h-9 px-4 py-2 text-sm",
    lg: "h-10 px-6 py-2 text-base"
  };
</script>

<button
  class={cn(
    "inline-flex items-center justify-center rounded-md font-medium transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-primary disabled:pointer-events-none disabled:opacity-50",
    variants[variant],
    sizes[size],
    className
  )}
  {disabled}
  on:click
>
  <slot />
</button>
