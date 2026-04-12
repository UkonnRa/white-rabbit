import { inject, watch, onMounted, onBeforeUnmount } from "vue";
import type { Ref } from "vue";
import { THEME_KEY } from "../themes/types";
import {
  setupRipple,
  type RippleHTMLElement,
} from "../themes/md3-expressive/ripple";

/**
 * Resolve a template ref to the underlying HTMLElement.
 * Handles both direct HTMLElement refs and Vue component instances (via $el).
 */
function resolveEl(
  ref: HTMLElement | { $el?: HTMLElement } | null | undefined,
): RippleHTMLElement | null {
  if (!ref) return null;
  if (ref instanceof HTMLElement) return ref as RippleHTMLElement;
  if ("$el" in ref && ref.$el instanceof HTMLElement)
    return ref.$el as RippleHTMLElement;
  return null;
}

/**
 * Applies the theme's ripple effect to a template ref element.
 *
 * When the active theme provides a ripple directive (e.g. md3-expressive),
 * the composable attaches the ripple pointerdown handler. When the theme
 * switches to one without ripple (e.g. tailwind-default), it cleans up.
 *
 * Base components stay theme-agnostic — they call `useRipple(elRef)` without
 * knowing whether the current theme uses ripple or not.
 */
export function useRipple(
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  elRef: Ref<any>,
) {
  const theme = inject(THEME_KEY);

  function hasRipple(): boolean {
    return !!theme?.value.directives?.ripple;
  }

  function attach() {
    const el = resolveEl(elRef.value);
    if (!el || el._rippleCleanup) return;
    if (!hasRipple()) return;
    setupRipple(el);
  }

  function detach() {
    const el = resolveEl(elRef.value);
    if (!el) return;
    el._rippleCleanup?.();
    el._rippleCleanup = undefined;
  }

  onMounted(() => {
    attach();
  });

  // React to theme changes
  if (theme) {
    watch(
      () => theme.value.name,
      () => {
        detach();
        attach();
      },
    );
  }

  onBeforeUnmount(() => {
    detach();
  });
}
