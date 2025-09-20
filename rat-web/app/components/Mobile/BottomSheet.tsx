import { createSignal, onMount, onCleanup, JSX, Show } from "solid-js";
import { Portal } from "solid-js/web";

interface BottomSheetProps {
  isOpen: boolean;
  onClose: () => void;
  children: JSX.Element;
  title?: string;
  snapPoints?: number[]; // Percentage heights (e.g., [25, 50, 90])
  defaultSnap?: number; // Index of default snap point
}

export function BottomSheet(props: BottomSheetProps) {
  const snapPoints = props.snapPoints || [50, 90];
  const defaultSnap = props.defaultSnap || 0;
  
  const [currentSnapIndex, setCurrentSnapIndex] = createSignal(defaultSnap);
  const [isDragging, setIsDragging] = createSignal(false);
  const [dragStartY, setDragStartY] = createSignal(0);
  const [sheetHeight, setSheetHeight] = createSignal(snapPoints[defaultSnap]);
  
  let sheetRef: HTMLDivElement | undefined;
  let startHeight = 0;
  
  const handleDragStart = (e: TouchEvent | MouseEvent) => {
    setIsDragging(true);
    const clientY = "touches" in e ? e.touches[0].clientY : e.clientY;
    setDragStartY(clientY);
    startHeight = sheetHeight();
    
    // Prevent text selection
    e.preventDefault();
  };
  
  const handleDragMove = (e: TouchEvent | MouseEvent) => {
    if (!isDragging()) return;
    
    const clientY = "touches" in e ? e.touches[0].clientY : e.clientY;
    const deltaY = dragStartY() - clientY;
    const viewportHeight = window.innerHeight;
    const newHeightPercent = startHeight + (deltaY / viewportHeight) * 100;
    
    // Clamp between 5% and 95%
    const clampedHeight = Math.max(5, Math.min(95, newHeightPercent));
    setSheetHeight(clampedHeight);
  };
  
  const handleDragEnd = () => {
    if (!isDragging()) return;
    setIsDragging(false);
    
    // Find closest snap point
    const currentHeight = sheetHeight();
    let closestIndex = 0;
    let closestDistance = Math.abs(currentHeight - snapPoints[0]);
    
    for (let i = 1; i < snapPoints.length; i++) {
      const distance = Math.abs(currentHeight - snapPoints[i]);
      if (distance < closestDistance) {
        closestDistance = distance;
        closestIndex = i;
      }
    }
    
    // If dragged down past 20% from bottom, close
    if (currentHeight < 20) {
      props.onClose();
      return;
    }
    
    // Snap to closest point
    setCurrentSnapIndex(closestIndex);
    setSheetHeight(snapPoints[closestIndex]);
  };
  
  const handleBackdropClick = (e: MouseEvent) => {
    if (e.target === e.currentTarget) {
      props.onClose();
    }
  };
  
  onMount(() => {
    document.addEventListener("mousemove", handleDragMove);
    document.addEventListener("mouseup", handleDragEnd);
    document.addEventListener("touchmove", handleDragMove, { passive: false });
    document.addEventListener("touchend", handleDragEnd);
    
    // Animate in
    setTimeout(() => {
      setSheetHeight(snapPoints[currentSnapIndex()]);
    }, 10);
  });
  
  onCleanup(() => {
    document.removeEventListener("mousemove", handleDragMove);
    document.removeEventListener("mouseup", handleDragEnd);
    document.removeEventListener("touchmove", handleDragMove);
    document.removeEventListener("touchend", handleDragEnd);
  });
  
  return (
    <Show when={props.isOpen}>
      <Portal>
        {/* Backdrop */}
        <div 
          class="fixed inset-0 bg-black/50 z-40 transition-opacity"
          onClick={handleBackdropClick}
        />
        
        {/* Sheet */}
        <div
          ref={sheetRef}
          class="fixed inset-x-0 bottom-0 bg-background border-t border-border rounded-t-3xl z-50 transition-transform"
          style={{
            height: `${sheetHeight()}vh`,
            transform: props.isOpen ? "translateY(0)" : "translateY(100%)",
            transition: isDragging() ? "none" : "height 0.3s cubic-bezier(0.4, 0, 0.2, 1), transform 0.3s cubic-bezier(0.4, 0, 0.2, 1)"
          }}
        >
          {/* Drag handle */}
          <div 
            class="drag-handle py-3 cursor-grab active:cursor-grabbing"
            onMouseDown={handleDragStart}
            onTouchStart={handleDragStart}
          >
            <div class="w-12 h-1 bg-muted-foreground/30 rounded-full mx-auto" />
          </div>
          
          {/* Header */}
          <Show when={props.title}>
            <div class="px-6 pb-4">
              <h2 class="text-lg font-semibold">{props.title}</h2>
            </div>
          </Show>
          
          {/* Content */}
          <div class="px-6 pb-6 safe overflow-y-auto" style={{ "max-height": "calc(100% - 60px)" }}>
            {props.children}
          </div>
        </div>
      </Portal>
    </Show>
  );
}