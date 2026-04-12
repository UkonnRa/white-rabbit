import type { Directive, DirectiveBinding } from "vue";

export interface RippleOptions {
  disabled?: boolean;
}

export interface RippleHTMLElement extends HTMLElement {
  _rippleCleanup?: () => void;
}

function createRipple(event: PointerEvent, el: HTMLElement) {
  const rect = el.getBoundingClientRect();
  const size = Math.max(rect.width, rect.height) * 2;
  const x = event.clientX - rect.left - size / 2;
  const y = event.clientY - rect.top - size / 2;

  const ripple = document.createElement("span");
  ripple.style.cssText = `
    position: absolute;
    border-radius: 50%;
    background: currentColor;
    opacity: 0.12;
    width: ${size}px;
    height: ${size}px;
    left: ${x}px;
    top: ${y}px;
    transform: scale(0);
    pointer-events: none;
    z-index: 0;
  `;

  // Use MD3 emphasized easing
  ripple.animate(
    [
      { transform: "scale(0)", opacity: 0.12 },
      { transform: "scale(1)", opacity: 0.08 },
    ],
    {
      duration: 450,
      easing: "cubic-bezier(0.05, 0.7, 0.1, 1)",
      fill: "forwards",
    },
  );

  el.appendChild(ripple);

  // Remove ripple on pointer up or leave
  const remove = () => {
    ripple.animate([{ opacity: 0.08 }, { opacity: 0 }], {
      duration: 200,
      easing: "ease-out",
      fill: "forwards",
    }).onfinish = () => ripple.remove();

    el.removeEventListener("pointerup", remove);
    el.removeEventListener("pointerleave", remove);
  };

  el.addEventListener("pointerup", remove, { once: true });
  el.addEventListener("pointerleave", remove, { once: true });
}

export function setupRipple(el: RippleHTMLElement) {
  // Ensure element can contain the absolute-positioned ripple
  const position = getComputedStyle(el).position;
  if (position === "static") {
    el.style.position = "relative";
  }
  el.style.overflow = "hidden";

  const handler = (e: Event) => createRipple(e as PointerEvent, el);
  el.addEventListener("pointerdown", handler);

  el._rippleCleanup = () => {
    el.removeEventListener("pointerdown", handler);
  };
}

function isDisabled(binding: DirectiveBinding<RippleOptions | boolean>) {
  if (typeof binding.value === "boolean") return !binding.value;
  if (binding.value && typeof binding.value === "object")
    return binding.value.disabled === true;
  return false;
}

/**
 * MD3 ripple directive.
 *
 * Usage:
 *   v-ripple              — enable ripple
 *   v-ripple="false"      — disable ripple
 *   v-ripple="{ disabled: true }" — disable ripple
 */
export const rippleDirective: Directive<
  RippleHTMLElement,
  RippleOptions | boolean
> = {
  mounted(el, binding) {
    if (!isDisabled(binding)) {
      setupRipple(el);
    }
  },
  updated(el, binding) {
    if (isDisabled(binding)) {
      el._rippleCleanup?.();
      el._rippleCleanup = undefined;
    } else if (!el._rippleCleanup) {
      setupRipple(el);
    }
  },
  unmounted(el) {
    el._rippleCleanup?.();
    el._rippleCleanup = undefined;
  },
};
