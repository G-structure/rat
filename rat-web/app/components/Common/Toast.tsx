import { createSignal, Show, onMount } from "solid-js";
import { Portal } from "solid-js/web";

interface ToastMessage {
  id: string;
  message: string;
  type?: "info" | "success" | "error" | "warning";
  duration?: number;
}

const [toasts, setToasts] = createSignal<ToastMessage[]>([]);

export function showToast(
  message: string, 
  type: ToastMessage["type"] = "info",
  duration = 3000
) {
  const id = Date.now().toString();
  const toast: ToastMessage = { id, message, type, duration };
  
  setToasts([...toasts(), toast]);
  
  if (duration > 0) {
    setTimeout(() => {
      removeToast(id);
    }, duration);
  }
}

function removeToast(id: string) {
  setToasts(toasts().filter(t => t.id !== id));
}

export function Toast() {
  const getIcon = (type: ToastMessage["type"]) => {
    switch (type) {
      case "success":
        return (
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
          </svg>
        );
      case "error":
        return (
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        );
      case "warning":
        return (
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
          </svg>
        );
      default:
        return (
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
        );
    }
  };
  
  const getColorClasses = (type: ToastMessage["type"]) => {
    switch (type) {
      case "success":
        return "bg-green-500/10 text-green-500 border-green-500/20";
      case "error":
        return "bg-red-500/10 text-red-500 border-red-500/20";
      case "warning":
        return "bg-yellow-500/10 text-yellow-500 border-yellow-500/20";
      default:
        return "bg-blue-500/10 text-blue-500 border-blue-500/20";
    }
  };
  
  return (
    <Show when={toasts().length > 0}>
      <Portal>
        <div class="fixed top-4 right-4 left-4 sm:left-auto sm:w-96 z-50 space-y-2 safe-top">
          {toasts().map(toast => (
            <div
              key={toast.id}
              class={`flex items-center gap-3 p-4 rounded-xl border backdrop-blur-ios animate-slide-up ${getColorClasses(toast.type)}`}
            >
              <div class="flex-shrink-0">
                {getIcon(toast.type)}
              </div>
              <p class="flex-1 text-sm font-medium">
                {toast.message}
              </p>
              <button
                onClick={() => removeToast(toast.id)}
                class="flex-shrink-0 p-1 rounded-lg hover:bg-white/10 transition-colors"
              >
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            </div>
          ))}
        </div>
      </Portal>
    </Show>
  );
}