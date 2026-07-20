import { animate, type JSAnimation } from "animejs";

/** Lightweight system motion tokens (synced with CSS vars). */
export const MOTION = {
  duration: 220,
  durationFast: 180,
  durationSlow: 280,
  ease: "outCubic",
  distance: 12,
  distanceTab: 8,
  distanceStack: 16,
} as const;

export type PresenceKind =
  | "fade"
  | "fade-up"
  | "fade-right"
  | "fade-down"
  | "tabbar";

type AnimHandle = JSAnimation;

const active = new WeakMap<Element, AnimHandle>();

export function prefersReducedMotion(): boolean {
  if (typeof window === "undefined" || typeof window.matchMedia !== "function") {
    return false;
  }
  return window.matchMedia("(prefers-reduced-motion: reduce)").matches;
}

export function motionDuration(base: number = MOTION.duration): number {
  if (prefersReducedMotion()) return 0;
  return base;
}

function cancelActive(el: Element) {
  const prev = active.get(el);
  if (prev) {
    try {
      prev.cancel();
    } catch {
      // ignore
    }
    active.delete(el);
  }
}

function run(
  el: Element,
  params: Record<string, unknown>,
  duration: number,
): Promise<void> {
  cancelActive(el);
  if (duration <= 0) {
    applyEndState(el, params);
    return Promise.resolve();
  }
  const animation = animate(el, {
    ...params,
    duration,
    ease: MOTION.ease,
  });
  active.set(el, animation);
  return animation.then(() => {
    if (active.get(el) === animation) active.delete(el);
  });
}

/** Best-effort apply final values when skipping animation. */
function applyEndState(el: Element, params: Record<string, unknown>) {
  const html = el as HTMLElement;
  for (const [key, value] of Object.entries(params)) {
    if (key === "duration" || key === "ease" || key === "delay") continue;
    const end = resolveEnd(value);
    if (end === undefined) continue;
    if (key === "opacity") {
      html.style.opacity = String(end);
    } else if (key === "translateY") {
      html.style.transform = `translateY(${formatPx(end)})`;
    } else if (key === "translateX") {
      html.style.transform = `translateX(${formatPx(end)})`;
    }
  }
}

function resolveEnd(value: unknown): string | number | undefined {
  if (Array.isArray(value)) return value[value.length - 1] as string | number;
  if (value && typeof value === "object" && "to" in (value as object)) {
    return (value as { to: string | number }).to;
  }
  if (typeof value === "number" || typeof value === "string") return value;
  return undefined;
}

function formatPx(v: string | number): string {
  return typeof v === "number" ? `${v}px` : v;
}

/**
 * Enter/leave presence for a single element (opacity + optional translate).
 * Use with Vue `<Transition :css="false">` hooks.
 */
export function animatePresence(
  el: Element,
  kind: PresenceKind,
  direction: "in" | "out",
  opts?: { duration?: number },
): Promise<void> {
  const base =
    opts?.duration ??
    (kind === "tabbar" || kind === "fade" ? MOTION.durationFast : MOTION.duration);
  const duration = motionDuration(base);
  const d =
    kind === "fade-right"
      ? MOTION.distanceStack
      : kind === "fade-up" || kind === "fade-down"
        ? MOTION.distance
        : kind === "tabbar"
          ? 10
          : MOTION.distanceTab;

  if (kind === "fade") {
    return run(
      el,
      {
        opacity: direction === "in" ? [0, 1] : [1, 0],
      },
      duration,
    );
  }

  if (kind === "fade-up") {
    return run(
      el,
      {
        opacity: direction === "in" ? [0, 1] : [1, 0],
        translateY: direction === "in" ? [d, 0] : [0, d * 0.5],
      },
      duration,
    );
  }

  if (kind === "fade-down") {
    return run(
      el,
      {
        opacity: direction === "in" ? [0, 1] : [1, 0],
        translateY: direction === "in" ? [-d, 0] : [0, -d * 0.5],
      },
      duration,
    );
  }

  if (kind === "fade-right") {
    return run(
      el,
      {
        opacity: direction === "in" ? [0, 1] : [1, 0],
        translateX: direction === "in" ? [d, 0] : [0, d * 0.75],
      },
      duration,
    );
  }

  // tabbar: slide down slightly when leaving
  return run(
    el,
    {
      opacity: direction === "in" ? [0, 1] : [1, 0],
      translateY: direction === "in" ? [d, 0] : [0, d],
    },
    duration,
  );
}

/** Confirm mask + card coordinated enter/leave. */
export function animateConfirm(
  maskEl: Element,
  direction: "in" | "out",
): Promise<void> {
  const duration = motionDuration(MOTION.duration);
  const card = maskEl.querySelector(".confirm-card");
  const d = MOTION.distance;

  cancelActive(maskEl);
  if (card) cancelActive(card);

  if (duration <= 0) {
    (maskEl as HTMLElement).style.opacity = direction === "in" ? "1" : "0";
    if (card) {
      (card as HTMLElement).style.opacity = direction === "in" ? "1" : "0";
      (card as HTMLElement).style.transform = "translateY(0px)";
    }
    return Promise.resolve();
  }

  const maskAnim = animate(maskEl, {
    opacity: direction === "in" ? [0, 1] : [1, 0],
    duration,
    ease: MOTION.ease,
  });
  active.set(maskEl, maskAnim);

  const cardPromise = card
    ? (() => {
        const cardAnim = animate(card, {
          opacity: direction === "in" ? [0, 1] : [1, 0],
          translateY: direction === "in" ? [d, 0] : [0, d * 0.6],
          duration,
          ease: MOTION.ease,
        });
        active.set(card, cardAnim);
        return cardAnim.then(() => {
          if (active.get(card) === cardAnim) active.delete(card);
        });
      })()
    : Promise.resolve();

  return Promise.all([
    maskAnim.then(() => {
      if (active.get(maskEl) === maskAnim) active.delete(maskEl);
    }),
    cardPromise,
  ]).then(() => undefined);
}

export function cancelMotion(el: Element | null | undefined) {
  if (el) cancelActive(el);
}
