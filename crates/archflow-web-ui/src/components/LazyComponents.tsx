/**
 * Lazy loading patterns for code splitting
 *
 * Provides lazy-loaded components for better initial load performance.
 * Architecture Reference: EPIC-WEB-009
 */

import { lazy, Suspense, type ComponentProps } from "react";
import { Skeleton } from "./Skeleton";

const DemoArchitectureLazy = lazy(async () => {
  const module = await import("./DemoArchitecture");
  return { default: module.DemoArchitecture };
});

type DemoArchitectureProps = ComponentProps<typeof DemoArchitectureLazy>;

export function LazyDemoArchitecture(props: DemoArchitectureProps) {
  return (
    <Suspense fallback={<Skeleton variant="rect" width="100%" height="100%" />}>
      <DemoArchitectureLazy {...props} />
    </Suspense>
  );
}
