import { createSignal, onMount, onCleanup, For, children, JSX } from "solid-js";

interface SwipeableViewsProps {
  children: JSX.Element;
  index?: number;
  onIndexChange?: (index: number) => void;
  threshold?: number;
  animateTransitions?: boolean;
}

export function SwipeableViews(props: SwipeableViewsProps) {
  const threshold = props.threshold || 50;
  const [currentIndex, setCurrentIndex] = createSignal(props.index || 0);
  const [translateX, setTranslateX] = createSignal(0);
  const [isDragging, setIsDragging] = createSignal(false);
  const [startX, setStartX] = createSignal(0);
  
  let containerRef: HTMLDivElement | undefined;
  let startTranslateX = 0;
  
  const childArray = children(() => props.children).toArray();
  const childCount = childArray.length;
  
  const handleStart = (clientX: number) => {
    setIsDragging(true);
    setStartX(clientX);
    startTranslateX = translateX();
  };
  
  const handleMove = (clientX: number) => {
    if (!isDragging() || !containerRef) return;
    
    const deltaX = clientX - startX();
    const containerWidth = containerRef.offsetWidth;
    const newTranslateX = startTranslateX + deltaX;
    
    // Add resistance at edges
    const maxTranslate = 0;
    const minTranslate = -(childCount - 1) * containerWidth;
    
    if (newTranslateX > maxTranslate || newTranslateX < minTranslate) {
      const overscroll = newTranslateX > maxTranslate 
        ? newTranslateX - maxTranslate 
        : minTranslate - newTranslateX;
      const resistance = 1 - Math.min(overscroll / containerWidth, 0.8);
      const resistedDelta = deltaX * resistance;
      setTranslateX(startTranslateX + resistedDelta);
    } else {
      setTranslateX(newTranslateX);
    }
  };
  
  const handleEnd = () => {
    if (!isDragging() || !containerRef) return;
    setIsDragging(false);
    
    const containerWidth = containerRef.offsetWidth;
    const deltaX = translateX() - startTranslateX;
    const currentTranslate = -currentIndex() * containerWidth;
    
    let newIndex = currentIndex();
    
    if (Math.abs(deltaX) > threshold) {
      if (deltaX < 0 && currentIndex() < childCount - 1) {
        newIndex = currentIndex() + 1;
      } else if (deltaX > 0 && currentIndex() > 0) {
        newIndex = currentIndex() - 1;
      }
    }
    
    setCurrentIndex(newIndex);
    setTranslateX(-newIndex * containerWidth);
    
    if (props.onIndexChange && newIndex !== currentIndex()) {
      props.onIndexChange(newIndex);
    }
  };
  
  const handleTouchStart = (e: TouchEvent) => {
    handleStart(e.touches[0].clientX);
  };
  
  const handleTouchMove = (e: TouchEvent) => {
    handleMove(e.touches[0].clientX);
    e.preventDefault(); // Prevent scrolling
  };
  
  const handleTouchEnd = () => {
    handleEnd();
  };
  
  const handleMouseDown = (e: MouseEvent) => {
    handleStart(e.clientX);
    e.preventDefault();
  };
  
  const handleMouseMove = (e: MouseEvent) => {
    handleMove(e.clientX);
  };
  
  const handleMouseUp = () => {
    handleEnd();
  };
  
  const handleResize = () => {
    if (!containerRef) return;
    const containerWidth = containerRef.offsetWidth;
    setTranslateX(-currentIndex() * containerWidth);
  };
  
  onMount(() => {
    window.addEventListener("resize", handleResize);
    document.addEventListener("mousemove", handleMouseMove);
    document.addEventListener("mouseup", handleMouseUp);
    
    // Set initial position
    if (containerRef) {
      const containerWidth = containerRef.offsetWidth;
      setTranslateX(-currentIndex() * containerWidth);
    }
  });
  
  onCleanup(() => {
    window.removeEventListener("resize", handleResize);
    document.removeEventListener("mousemove", handleMouseMove);
    document.removeEventListener("mouseup", handleMouseUp);
  });
  
  // Update position when index prop changes
  if (props.index !== undefined && props.index !== currentIndex()) {
    setCurrentIndex(props.index);
    if (containerRef) {
      const containerWidth = containerRef.offsetWidth;
      setTranslateX(-props.index * containerWidth);
    }
  }
  
  return (
    <div class="relative overflow-hidden h-full">
      <div
        ref={containerRef}
        class="flex h-full"
        style={{
          transform: `translateX(${translateX()}px)`,
          transition: isDragging() || !props.animateTransitions 
            ? "none" 
            : "transform 0.3s cubic-bezier(0.4, 0, 0.2, 1)"
        }}
        onTouchStart={handleTouchStart}
        onTouchMove={handleTouchMove}
        onTouchEnd={handleTouchEnd}
        onMouseDown={handleMouseDown}
      >
        <For each={childArray}>
          {(child) => (
            <div class="w-full h-full flex-shrink-0">
              {child}
            </div>
          )}
        </For>
      </div>
      
      {/* Page indicators */}
      <div class="absolute bottom-4 left-0 right-0 flex justify-center gap-2">
        <For each={new Array(childCount)}>
          {(_, index) => (
            <button
              onClick={() => {
                setCurrentIndex(index());
                if (containerRef) {
                  const containerWidth = containerRef.offsetWidth;
                  setTranslateX(-index() * containerWidth);
                }
                if (props.onIndexChange) {
                  props.onIndexChange(index());
                }
              }}
              class={`w-2 h-2 rounded-full transition-all ${
                currentIndex() === index() 
                  ? "bg-primary w-6" 
                  : "bg-muted-foreground/30"
              }`}
            />
          )}
        </For>
      </div>
    </div>
  );
}