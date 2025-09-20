import { Show } from "solid-js";

interface LoadingProps {
  size?: "sm" | "md" | "lg";
  message?: string;
  fullscreen?: boolean;
}

export function Loading(props: LoadingProps) {
  const sizeClasses = {
    sm: "w-4 h-4",
    md: "w-8 h-8",
    lg: "w-12 h-12"
  };
  
  const size = props.size || "md";
  
  const spinner = (
    <div class="relative">
      <div class={`${sizeClasses[size]} relative`}>
        <div class="absolute inset-0 rounded-full border-2 border-primary/20"></div>
        <div class="absolute inset-0 rounded-full border-2 border-primary border-t-transparent animate-spin"></div>
      </div>
    </div>
  );
  
  if (props.fullscreen) {
    return (
      <div class="fixed inset-0 bg-background/80 backdrop-blur-sm z-50 flex items-center justify-center">
        <div class="text-center space-y-4">
          {spinner}
          <Show when={props.message}>
            <p class="text-sm text-muted-foreground">{props.message}</p>
          </Show>
        </div>
      </div>
    );
  }
  
  return (
    <div class="flex flex-col items-center justify-center gap-4">
      {spinner}
      <Show when={props.message}>
        <p class="text-sm text-muted-foreground">{props.message}</p>
      </Show>
    </div>
  );
}

// Shimmer loading placeholder
export function Shimmer(props: { class?: string }) {
  return (
    <div class={`shimmer bg-muted rounded ${props.class || ""}`} />
  );
}

// Skeleton loader for text
export function SkeletonText(props: { lines?: number; class?: string }) {
  const lines = props.lines || 1;
  
  return (
    <div class={`space-y-2 ${props.class || ""}`}>
      {Array.from({ length: lines }).map((_, i) => (
        <Shimmer 
          class={`h-4 ${i === lines - 1 ? "w-3/4" : "w-full"}`} 
        />
      ))}
    </div>
  );
}

// Skeleton loader for cards
export function SkeletonCard(props: { class?: string }) {
  return (
    <div class={`p-4 rounded-xl bg-secondary/50 space-y-3 ${props.class || ""}`}>
      <div class="flex items-center gap-3">
        <Shimmer class="w-10 h-10 rounded-full" />
        <div class="flex-1 space-y-2">
          <Shimmer class="h-4 w-1/3" />
          <Shimmer class="h-3 w-1/2" />
        </div>
      </div>
      <SkeletonText lines={2} />
    </div>
  );
}