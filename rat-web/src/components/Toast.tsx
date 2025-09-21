import { Show, createEffect, onCleanup } from "solid-js";

type ToastType = "info" | "success" | "error" | "warning";

interface ToastProps {
  message: string;
  type?: ToastType;
  duration?: number;
  onClose?: () => void;
}

const typeStyles: Record<ToastType, string> = {
  info: "bg-slate-800 text-white",
  success: "bg-emerald-600 text-white",
  error: "bg-red-600 text-white",
  warning: "bg-amber-500 text-black",
};

export function Toast(props: ToastProps) {
  const lifespan = () => props.duration ?? 5000;

  createEffect(() => {
    const ms = lifespan();
    if (ms <= 0) return;

    const timer = setTimeout(() => {
      props.onClose?.();
    }, ms);

    onCleanup(() => clearTimeout(timer));
  });

  const tone = () => typeStyles[props.type || "info"];

  return (
    <div class="fixed inset-x-0 bottom-0 px-4 pb-8 flex justify-center pointer-events-none" role="status" aria-live="polite">
      <div class={`pointer-events-auto flex items-center gap-3 rounded-2xl px-4 py-3 shadow-xl ${tone()}`}>
        <span class="text-sm font-medium">{props.message}</span>
        <Show when={props.onClose}>
          <button
            type="button"
            class="rounded-full bg-black/10 px-2 py-1 text-xs font-semibold uppercase tracking-wide"
            onClick={() => props.onClose?.()}
          >
            Close
          </button>
        </Show>
      </div>
    </div>
  );
}
