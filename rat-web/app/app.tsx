import { MetaProvider } from "@solidjs/meta";
import { Router, Route, Navigate, useNavigate } from "@solidjs/router";
import { Suspense, lazy, Show, createEffect, createSignal, ParentComponent } from "solid-js";
import { QueryClient, QueryClientProvider } from "@tanstack/solid-query";
import { Toast } from "./components/Common/Toast";
import { Loading } from "./components/Common/Loading";
import { authState, checkAuth } from "~/stores/authStore";
import "./styles/globals.css";

// Lazy load routes
const Home = lazy(() => import("./routes/index"));
const Login = lazy(() => import("./routes/login"));
const Onboarding = lazy(() => import("./routes/onboarding"));
const Dashboard = lazy(() => import("./routes/dashboard"));
const CoreApp = lazy(() => import("./routes/core-app"));
const RepoView = lazy(() => import("./routes/repos/[...slug]"));

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      retry: 1,
      staleTime: 5 * 60 * 1000, // 5 minutes
      refetchOnWindowFocus: false,
      refetchOnMount: "always"
    }
  }
});

// Protected Route Component
const ProtectedRoute: ParentComponent = (props) => {
  const navigate = useNavigate();
  const [isChecking, setIsChecking] = createSignal(true);
  
  createEffect(async () => {
    // In development mode, skip auth checks entirely
    if (import.meta.env.DEV) {
      setIsChecking(false);
      return;
    }
    
    const authenticated = await checkAuth();
    setIsChecking(false);
    
    // In production, require authentication
    if (!authenticated) {
      navigate("/onboarding", { replace: true });
    }
  });
  
  return (
    <Show when={!isChecking()} fallback={<Loading fullscreen message="Checking authentication..." />}>
      {props.children}
    </Show>
  );
};

export default function App() {
  return (
    <MetaProvider>
      <QueryClientProvider client={queryClient}>
        <Router>
          <Suspense fallback={<Loading fullscreen message="Loading..." />}>
            <Route path="/" component={() => <Navigate href="/onboarding" />} />
            <Route path="/login" component={() => <Navigate href="/onboarding" />} />
            <Route path="/onboarding" component={Onboarding} />
            <Route path="/dashboard" component={Dashboard} />
            <Route path="/app" component={CoreApp} />
            <Route path="/repos/*" component={RepoView} />
          </Suspense>
          <Toast />
        </Router>
      </QueryClientProvider>
    </MetaProvider>
  );
}