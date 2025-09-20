import { mount, StartClient } from "solid-start/entry-client";

mount(() => <StartClient />, document);

// Register service worker
if ("serviceWorker" in navigator && import.meta.env.PROD) {
  window.addEventListener("load", () => {
    navigator.serviceWorker
      .register("/sw.js")
      .then((registration) => {
        console.log("SW registered:", registration);
        
        // Check for updates periodically
        setInterval(() => {
          registration.update();
        }, 60 * 60 * 1000); // Check every hour
      })
      .catch((error) => {
        console.log("SW registration failed:", error);
      });
  });
}

// Handle PWA install prompt
let deferredPrompt: BeforeInstallPromptEvent | null = null;

interface BeforeInstallPromptEvent extends Event {
  prompt: () => Promise<void>;
  userChoice: Promise<{ outcome: "accepted" | "dismissed"; platform: string }>;
}

window.addEventListener("beforeinstallprompt", (e: Event) => {
  // Prevent Chrome 67 and earlier from automatically showing the prompt
  e.preventDefault();
  // Stash the event so it can be triggered later
  deferredPrompt = e as BeforeInstallPromptEvent;
  
  // Show custom install UI
  const installPrompt = document.getElementById("install-prompt");
  if (installPrompt) {
    installPrompt.classList.remove("hidden");
  }
});

// Handle app installed event
window.addEventListener("appinstalled", () => {
  // Clear the deferredPrompt so it can be garbage collected
  deferredPrompt = null;
  
  // Hide install UI
  const installPrompt = document.getElementById("install-prompt");
  if (installPrompt) {
    installPrompt.classList.add("hidden");
  }
  
  console.log("PWA was installed");
});

// Export install function for use in components
declare global {
  interface Window {
    installPWA: () => Promise<void>;
  }
}

window.installPWA = async () => {
  if (!deferredPrompt) {
    console.log("Install prompt not available");
    return;
  }
  
  // Show the install prompt
  deferredPrompt.prompt();
  
  // Wait for the user to respond to the prompt
  const { outcome } = await deferredPrompt.userChoice;
  console.log(`User response to the install prompt: ${outcome}`);
  
  // Clear the deferred prompt
  deferredPrompt = null;
};