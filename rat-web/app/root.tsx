// @refresh reload
import { Suspense } from "solid-js";
import {
  Body,
  ErrorBoundary,
  FileRoutes,
  Head,
  Html,
  Link,
  Meta,
  Routes,
  Scripts,
  Title,
} from "solid-start";
import { MetaProvider } from "@solidjs/meta";
import "./styles/globals.css";

export default function Root() {
  return (
    <Html lang="en" class="dark">
      <Head>
        <Meta charset="utf-8" />
        <Meta name="viewport" content="width=device-width, initial-scale=1, viewport-fit=cover" />
        <Meta name="description" content="AI-powered mobile code editor with GitHub integration" />
        <Meta name="apple-mobile-web-app-capable" content="yes" />
        <Meta name="apple-mobile-web-app-status-bar-style" content="black-translucent" />
        <Meta name="theme-color" content="#0b0b0b" />
        <Link rel="manifest" href="/manifest.webmanifest" />
        <Link rel="icon" type="image/png" sizes="32x32" href="/icons/favicon-32.png" />
        <Link rel="icon" type="image/png" sizes="16x16" href="/icons/favicon-16.png" />
        <Link rel="apple-touch-icon" sizes="180x180" href="/icons/apple-touch-icon.png" />
        
        {/* Preconnect to external domains */}
        <Link rel="preconnect" href="https://api.github.com" />
        <Link rel="preconnect" href="https://raw.githubusercontent.com" />
        
        {/* PWA splash screens for iOS */}
        <Link rel="apple-touch-startup-image" href="/splash/iphone-se.png" 
              media="(device-width: 375px) and (device-height: 667px) and (-webkit-device-pixel-ratio: 2)" />
        <Link rel="apple-touch-startup-image" href="/splash/iphone-xr.png" 
              media="(device-width: 414px) and (device-height: 896px) and (-webkit-device-pixel-ratio: 2)" />
        <Link rel="apple-touch-startup-image" href="/splash/iphone-x.png" 
              media="(device-width: 375px) and (device-height: 812px) and (-webkit-device-pixel-ratio: 3)" />
        <Link rel="apple-touch-startup-image" href="/splash/iphone-14.png" 
              media="(device-width: 390px) and (device-height: 844px) and (-webkit-device-pixel-ratio: 3)" />
        <Link rel="apple-touch-startup-image" href="/splash/iphone-14-pro-max.png" 
              media="(device-width: 430px) and (device-height: 932px) and (-webkit-device-pixel-ratio: 3)" />
        
        <Title>RAT Mobile IDE</Title>
      </Head>
      <Body>
        <ErrorBoundary>
          <Suspense>
            <MetaProvider>
              <Routes>
                <FileRoutes />
              </Routes>
            </MetaProvider>
          </Suspense>
        </ErrorBoundary>
        <Scripts />
      </Body>
    </Html>
  );
}