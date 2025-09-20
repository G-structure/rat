import { createSignal, createEffect } from "solid-js";

export function useLocalStorage<T>(key: string, initialValue: T) {
  // Get initial value from localStorage or use provided initial value
  const getStoredValue = () => {
    try {
      const item = window.localStorage.getItem(key);
      return item ? JSON.parse(item) : initialValue;
    } catch (error) {
      console.error(`Error reading localStorage key "${key}":`, error);
      return initialValue;
    }
  };

  const [value, setValue] = createSignal<T>(getStoredValue());

  // Update localStorage when value changes
  createEffect(() => {
    try {
      window.localStorage.setItem(key, JSON.stringify(value()));
    } catch (error) {
      console.error(`Error setting localStorage key "${key}":`, error);
    }
  });

  return [value, setValue] as const;
}