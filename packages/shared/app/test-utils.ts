import { createApp, defineComponent, h } from "vue";

/**
 * Run a composable inside a minimal Vue app so that provide/inject,
 * computed, watchEffect etc. work correctly.
 *
 * Returns the composable's return value after the app is mounted.
 */
export function withSetup<T>(composable: () => T): {
  result: T;
  unmount: () => void;
} {
  let result!: T;
  const app = createApp(
    defineComponent({
      setup() {
        result = composable();
        return () => h("div");
      },
    }),
  );
  const root = document.createElement("div");
  document.body.appendChild(root);
  app.mount(root);
  return {
    result,
    unmount: () => {
      app.unmount();
      root.remove();
    },
  };
}

/**
 * Run a parent composable that calls provide(), then run a child
 * composable that calls inject() — in a proper parent→child tree
 * so Vue's provide/inject chain works correctly.
 */
export function withNestedSetup<P, C>(
  parent: () => P,
  child: () => C,
): { parentResult: P; childResult: C; unmount: () => void } {
  let parentResult!: P;
  let childResult!: C;

  const Child = defineComponent({
    setup() {
      childResult = child();
      return () => h("div");
    },
  });

  const Parent = defineComponent({
    setup() {
      parentResult = parent();
      return () => h(Child);
    },
  });

  const app = createApp(Parent);
  const root = document.createElement("div");
  document.body.appendChild(root);
  app.mount(root);

  return {
    parentResult,
    childResult,
    unmount: () => {
      app.unmount();
      root.remove();
    },
  };
}
