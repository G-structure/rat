import { createSignal } from "solid-js";

export function SimpleLiquidChat() {
  const [isOpen, setIsOpen] = createSignal(false);
  
  return (
    <>
      {/* Test Button */}
      <button
        onClick={() => setIsOpen(!isOpen())}
        class="fixed bottom-6 right-6 w-14 h-14 rounded-full shadow-2xl flex items-center justify-center transition-all duration-300 hover:scale-105 active:scale-95 z-40 bg-gradient-to-br from-blue-500 via-purple-500 to-pink-500"
      >
        <svg class="w-6 h-6 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 10h.01M12 10h.01M16 10h.01M9 16H5a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v8a2 2 0 01-2 2h-5l-5 5v-5z" />
        </svg>
      </button>
      
      {/* Simple Modal Test */}
      {isOpen() && (
        <div class="fixed inset-0 z-50 bg-black/50">
          <div class="absolute bottom-0 left-0 right-0 h-1/2 bg-white/10 backdrop-blur-xl p-4">
            <h2 class="text-white text-xl">Liquid Chat Test</h2>
            <button onClick={() => setIsOpen(false)} class="text-white">Close</button>
          </div>
        </div>
      )}
    </>
  );
}