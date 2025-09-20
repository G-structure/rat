import { MetaProvider } from "@solidjs/meta";
import { Router, Route } from "@solidjs/router";
import { Suspense, lazy } from "solid-js";
import { QueryClient, QueryClientProvider } from "@tanstack/solid-query";
import { Toast } from "./components/Common/Toast";
import { Loading } from "./components/Common/Loading";
import "./styles/globals.css";

// Lazy load routes
const Home = lazy(() => import("./routes/index"));
const Login = lazy(() => import("./routes/login"));
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

export default function App() {
  return (
    <MetaProvider>
      <QueryClientProvider client={queryClient}>
        <Router>
          <Suspense fallback={<Loading fullscreen message="Loading..." />}>
            <Route path="/" component={Home} />
            <Route path="/login" component={Login} />
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