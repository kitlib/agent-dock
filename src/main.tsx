import React, { lazy, Suspense, useEffect } from "react";
import ReactDOM from "react-dom/client";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { getCurrentWindow } from "@tauri-apps/api/window";
import "./index.css";
import "./i18n";

const HomePage = lazy(() => import("./pages/home"));
const AboutPage = lazy(() => import("./pages/about"));
const SettingsPage = lazy(() => import("./pages/settings"));

const pageMap = {
  "/": HomePage,
  "/about": AboutPage,
  "/settings": SettingsPage,
};

function getPageComponent(pathname: string) {
  return pageMap[pathname as keyof typeof pageMap] ?? HomePage;
}

const pathname = window.location.pathname;
const PageComponent = getPageComponent(pathname);

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      refetchOnWindowFocus: false,
      retry: 0,
    },
  },
});

function AppWrapper() {
  useEffect(() => {
    getCurrentWindow().show();
  }, []);

  return <PageComponent />;
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <QueryClientProvider client={queryClient}>
      <Suspense fallback={null}>
        <AppWrapper />
      </Suspense>
    </QueryClientProvider>
  </React.StrictMode>
);
