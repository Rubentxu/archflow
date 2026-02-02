/**
 * Lazy loading patterns for code splitting
 *
 * Provides lazy-loaded components for better initial load performance.
 * Architecture Reference: EPIC-WEB-009
 */

import { lazy, Suspense } from "react";
import { Skeleton } from "./Skeleton";

export const DemoArchitecture = lazy(
  () => import("./DemoArchitecture").then((m) => ({
    default: m.DemoArchitecture,
  })),
);

export const LazyDemoArchitecture = (props: unknown) => (
  <Suspense fallback={<Skeleton variant="rect" width="100%" height="100%" />}>
    <DemoArchitecture {...props} />
  </Suspense>
);
