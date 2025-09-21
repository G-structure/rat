import { MetaProvider } from "@solidjs/meta";
import { Router, Route, Navigate, useNavigate } from "@solidjs/router";
import { Suspense, lazy, Show, createEffect, createSignal, ParentComponent } from "solid-js";
import { QueryClient, QueryClientProvider } from "@tanstack/solid-query";
import { Toast } from "./components/Common/Toast";
import { Loading } from "./components/Common/Loading";
import { authState, checkAuth } from "../src/stores/authStore";
import "./styles/globals.css";

// Lazy load routes
const Home = lazy(() => import("./routes/index"));
const Login = lazy(() => import("./routes/login"));
const Onboarding = lazy(() => import("../src/routes/onboarding"));
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
    const authenticated = await checkAuth();
    setIsChecking(false);
    
    if (!authenticated) {
      navigate("/onboarding", { replace: true });
    }
  });
  
  return (
    <Show when={!isChecking()} fallback={<Loading fullscreen message="Checking authentication..." />}>
      <Show when={authState.isAuthenticated} fallback={<Navigate href="/onboarding" />}>
        {props.children}
      </Show>
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
            <Route path="/dashboard" component={() => <ProtectedRoute><Dashboard /></ProtectedRoute>} />
            <Route path="/app" component={() => <ProtectedRoute><CoreApp /></ProtectedRoute>} />
            <Route path="/repos/*" component={() => <ProtectedRoute><RepoView /></ProtectedRoute>} />
          </Suspense>
          <Toast />
        </Router>
      </QueryClientProvider>
    </MetaProvider>
  );
}