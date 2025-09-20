import { createSignal, For, Show, onMount, onCleanup } from "solid-js";
import { Portal } from "solid-js/web";

interface ChatInstance {
  id: string;
  content: string;
  status: 'idle' | 'generating' | 'ready';
  messages: Array<{ role: 'user' | 'assistant'; content: string }>;
}

interface LiquidChatModalProps {
  isOpen: boolean;
  onClose: () => void;
}

export function LiquidChatModal(props: LiquidChatModalProps) {
  const MAX_INSTANCES = 8;
  const [chatInstances, setChatInstances] = createSignal<ChatInstance[]>([
    { id: '1', content: '', status: 'idle', messages: [] }
  ]);
  const [activeInstanceId, setActiveInstanceId] = createSignal('1');
  const [swipeStartX, setSwipeStartX] = createSignal(0);
  const [swipeX, setSwipeX] = createSignal(0);
  const [isAnimating, setIsAnimating] = createSignal(false);
  
  let modalRef: HTMLDivElement | undefined;
  let inputRef: HTMLTextAreaElement | undefined;
  
  const activeInstance = () => {
    return chatInstances().find(instance => instance.id === activeInstanceId());
  };
  
  const getInstanceColor = (status: ChatInstance['status']) => {
    switch (status) {
      case 'generating':
        return 'bg-gradient-to-br from-yellow-400/20 via-yellow-300/20 to-white/10';
      case 'ready':
        return 'bg-gradient-to-br from-green-500/20 via-green-400/20 to-green-300/10';
      default:
        return 'bg-gradient-to-br from-gray-500/20 via-gray-400/20 to-gray-300/10';
    }
  };
  
  const getDotColor = (instance: ChatInstance) => {
    if (instance.status === 'generating') {
      return 'bg-yellow-400 animate-pulse shadow-[0_0_8px_rgba(250,204,21,0.6)]';
    }
    if (instance.status === 'ready' && instance.messages.length > 0) {
      return 'bg-green-500 shadow-[0_0_8px_rgba(34,197,94,0.6)]';
    }
    return 'bg-gray-400';
  };
  
  const createNewInstance = () => {
    if (chatInstances().length >= MAX_INSTANCES) return;
    
    const newId = String(Math.max(...chatInstances().map(i => parseInt(i.id))) + 1);
    setChatInstances([...chatInstances(), {
      id: newId,
      content: '',
      status: 'idle',
      messages: []
    }]);
    setActiveInstanceId(newId);
  };
  
  const removeInstance = (id: string) => {
    if (chatInstances().length === 1) return;
    
    const instances = chatInstances().filter(i => i.id !== id);
    setChatInstances(instances);
    
    if (activeInstanceId() === id) {
      setActiveInstanceId(instances[0].id);
    }
  };
  
  const updateInstanceContent = (content: string) => {
    setChatInstances(instances =>
      instances.map(instance =>
        instance.id === activeInstanceId()
          ? { ...instance, content }
          : instance
      )
    );
  };
  
  const sendMessage = async () => {
    const instance = activeInstance();
    if (!instance || !instance.content.trim()) return;
    
    // Add user message
    setChatInstances(instances =>
      instances.map(inst =>
        inst.id === activeInstanceId()
          ? {
              ...inst,
              messages: [...inst.messages, { role: 'user', content: inst.content }],
              content: '',
              status: 'generating'
            }
          : inst
      )
    );
    
    // Simulate AI response
    setTimeout(() => {
      setChatInstances(instances =>
        instances.map(inst =>
          inst.id === activeInstanceId()
            ? {
                ...inst,
                messages: [...inst.messages, {
                  role: 'assistant',
                  content: `I understand you want to ${instance.content}. Here's how we can approach this...`
                }],
                status: 'ready'
              }
            : inst
        )
      );
    }, 2000);
  };
  
  // Touch handling for swipe navigation
  const handleTouchStart = (e: TouchEvent) => {
    setSwipeStartX(e.touches[0].clientX);
    setIsAnimating(false);
  };
  
  const handleTouchMove = (e: TouchEvent) => {
    const currentX = e.touches[0].clientX;
    const diff = currentX - swipeStartX();
    setSwipeX(diff);
  };
  
  const handleTouchEnd = () => {
    const threshold = 80;
    const currentIndex = chatInstances().findIndex(i => i.id === activeInstanceId());
    
    setIsAnimating(true);
    
    if (swipeX() > threshold && currentIndex > 0) {
      setActiveInstanceId(chatInstances()[currentIndex - 1].id);
    } else if (swipeX() < -threshold && currentIndex < chatInstances().length - 1) {
      setActiveInstanceId(chatInstances()[currentIndex + 1].id);
    }
    
    setSwipeX(0);
  };
  
  // Keyboard shortcuts
  const handleKeyDown = (e: KeyboardEvent) => {
    if (!props.isOpen) return;
    
    if (e.key === 'Enter' && (e.metaKey || e.ctrlKey)) {
      e.preventDefault();
      sendMessage();
    } else if (e.key === 'Escape') {
      props.onClose();
    } else if (e.key === 'Tab' && e.ctrlKey) {
      e.preventDefault();
      const currentIndex = chatInstances().findIndex(i => i.id === activeInstanceId());
      const nextIndex = e.shiftKey 
        ? (currentIndex - 1 + chatInstances().length) % chatInstances().length
        : (currentIndex + 1) % chatInstances().length;
      setActiveInstanceId(chatInstances()[nextIndex].id);
    }
  };
  
  onMount(() => {
    if (modalRef) {
      modalRef.addEventListener('touchstart', handleTouchStart);
      modalRef.addEventListener('touchmove', handleTouchMove);
      modalRef.addEventListener('touchend', handleTouchEnd);
    }
    
    document.addEventListener('keydown', handleKeyDown);
    
    if (inputRef) {
      inputRef.focus();
    }
  });
  
  onCleanup(() => {
    if (modalRef) {
      modalRef.removeEventListener('touchstart', handleTouchStart);
      modalRef.removeEventListener('touchmove', handleTouchMove);
      modalRef.removeEventListener('touchend', handleTouchEnd);
    }
    
    document.removeEventListener('keydown', handleKeyDown);
  });
  
  return (
    <Show when={props.isOpen}>
      <Portal>
        <div class="fixed inset-0 z-50 overflow-hidden">
          {/* Backdrop (click top half to close) */}
          <div 
            class="absolute inset-x-0 top-0 h-1/2 bg-black/30 backdrop-blur-sm transition-opacity"
            onClick={props.onClose}
          />
          
          {/* Chat Modal (bottom half) */}
          <div 
            ref={modalRef}
            class="absolute inset-x-0 bottom-0 h-1/2 transform transition-transform duration-300"
            style={{
              transform: `translateX(${isAnimating() ? 0 : swipeX()}px)`,
              transition: isAnimating() ? 'transform 0.3s ease-out' : 'none'
            }}
          >
            {/* Glass morphism container */}
            <div class="h-full bg-gradient-to-t from-black/90 via-black/80 to-black/60 backdrop-blur-2xl border-t border-white/10 rounded-t-3xl shadow-2xl">
              {/* Drag handle */}
              <div class="flex justify-center pt-3 pb-2">
                <div class="w-12 h-1 bg-white/30 rounded-full" />
              </div>
              
              {/* Instance dots navigation */}
              <div class="flex items-center justify-center gap-3 px-4 pb-3">
                <For each={chatInstances()}>
                  {(instance) => (
                    <button
                      onClick={() => setActiveInstanceId(instance.id)}
                      class={`relative w-3 h-3 rounded-full transition-all duration-300 ${getDotColor(instance)} ${
                        instance.id === activeInstanceId() ? 'scale-125' : 'scale-100 opacity-60'
                      }`}
                      title={`Chat ${instance.id}`}
                    >
                      <Show when={instance.id === activeInstanceId() && instance.status === 'generating'}>
                        <div class="absolute inset-0 rounded-full bg-yellow-400 animate-ping opacity-75" />
                      </Show>
                    </button>
                  )}
                </For>
                <Show when={chatInstances().length < MAX_INSTANCES}>
                  <button
                    onClick={createNewInstance}
                    class="w-3 h-3 rounded-full border border-white/30 hover:bg-white/20 transition-colors"
                    title="New chat"
                  >
                    <span class="sr-only">Add new chat</span>
                  </button>
                </Show>
              </div>
              
              {/* Active chat container */}
              <div class={`h-[calc(100%-5rem)] px-4 pb-4 rounded-2xl mx-3 transition-colors duration-500 ${
                getInstanceColor(activeInstance()?.status || 'idle')
              }`}>
                <div class="h-full flex flex-col">
                  {/* Messages */}
                  <div class="flex-1 overflow-y-auto space-y-3 mb-3 scrollbar-thin">
                    <For each={activeInstance()?.messages || []}>
                      {(message) => (
                        <div class={`flex ${message.role === 'user' ? 'justify-end' : 'justify-start'}`}>
                          <div class={`max-w-[80%] rounded-2xl px-4 py-2 ${
                            message.role === 'user'
                              ? 'bg-blue-500/20 text-blue-100 border border-blue-400/30'
                              : 'bg-white/10 text-gray-100 border border-white/10'
                          }`}>
                            <p class="text-sm whitespace-pre-wrap">{message.content}</p>
                          </div>
                        </div>
                      )}
                    </For>
                    <Show when={activeInstance()?.status === 'generating'}>
                      <div class="flex justify-start">
                        <div class="bg-white/10 rounded-2xl px-4 py-3 border border-white/10">
                          <div class="flex gap-1">
                            <div class="w-2 h-2 bg-white/60 rounded-full animate-bounce" style="animation-delay: 0ms" />
                            <div class="w-2 h-2 bg-white/60 rounded-full animate-bounce" style="animation-delay: 150ms" />
                            <div class="w-2 h-2 bg-white/60 rounded-full animate-bounce" style="animation-delay: 300ms" />
                          </div>
                        </div>
                      </div>
                    </Show>
                  </div>
                  
                  {/* Input area */}
                  <div class="relative">
                    <textarea
                      ref={inputRef}
                      value={activeInstance()?.content || ''}
                      onInput={(e) => updateInstanceContent(e.currentTarget.value)}
                      onKeyDown={(e) => {
                        if (e.key === 'Enter' && (e.metaKey || e.ctrlKey)) {
                          e.preventDefault();
                          sendMessage();
                        }
                      }}
                      placeholder="Ask anything..."
                      class="w-full px-4 py-3 pr-12 bg-white/5 border border-white/10 rounded-2xl resize-none focus:outline-none focus:border-white/30 text-white placeholder-white/40 transition-colors"
                      rows="2"
                      disabled={activeInstance()?.status === 'generating'}
                    />
                    <button
                      onClick={sendMessage}
                      disabled={!activeInstance()?.content.trim() || activeInstance()?.status === 'generating'}
                      class="absolute right-2 bottom-2 w-8 h-8 rounded-lg bg-white/10 hover:bg-white/20 disabled:opacity-30 disabled:cursor-not-allowed transition-colors flex items-center justify-center"
                    >
                      <svg class="w-5 h-5 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 19l9 2-9-18-9 18 9-2zm0 0v-8" />
                      </svg>
                    </button>
                  </div>
                  
                  {/* Hint text */}
                  <div class="flex items-center justify-between mt-2 text-xs text-white/40">
                    <span>⌘+Enter to send • Ctrl+Tab to switch chats</span>
                    <Show when={chatInstances().length > 1}>
                      <button
                        onClick={() => removeInstance(activeInstanceId())}
                        class="hover:text-red-400 transition-colors"
                      >
                        Close chat
                      </button>
                    </Show>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </Portal>
    </Show>
  );
}