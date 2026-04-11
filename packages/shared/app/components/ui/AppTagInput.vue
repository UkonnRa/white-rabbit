<script setup lang="ts">
import { ref, computed } from "vue";
import {
  TagsInputRoot,
  TagsInputInput,
  TagsInputItem,
  TagsInputItemText,
  TagsInputItemDelete,
  ComboboxRoot,
  ComboboxInput,
  ComboboxPortal,
  ComboboxContent,
  ComboboxViewport,
  ComboboxItem,
  ComboboxItemIndicator,
  ComboboxEmpty,
} from "reka-ui";
import { resolveRecipe } from "../../composables/useRecipe";
import { useTheme } from "../../composables/useTheme";
import AppIcon from "./AppIcon.vue";

const props = withDefaults(
  defineProps<{
    size?: "sm" | "md" | "lg";
    disabled?: boolean;
    placeholder?: string;
    /** Available suggestions shown in the dropdown. */
    suggestions?: string[];
  }>(),
  {
    size: "md",
    placeholder: "Add tag…",
    suggestions: () => [],
  },
);

const model = defineModel<string[]>({ default: () => [] });
const { theme } = useTheme();

const searchTerm = ref("");

const filteredSuggestions = computed(() => {
  const term = searchTerm.value.toLowerCase().trim();
  const selected = new Set(model.value);
  return props.suggestions.filter(
    (s) => !selected.has(s) && (!term || s.toLowerCase().includes(term)),
  );
});

const showCreateOption = computed(() => {
  const term = searchTerm.value.trim();
  if (!term) return false;
  // Don't show "create" if the term already exists in model or suggestions
  const lower = term.toLowerCase();
  if (model.value.some((t) => t.toLowerCase() === lower)) return false;
  if (props.suggestions.some((s) => s.toLowerCase() === lower)) return false;
  return true;
});

function handleSelect(val: string) {
  if (!model.value.includes(val)) {
    model.value = [...model.value, val];
  }
  searchTerm.value = "";
}

function cls(variant: string, extra?: Record<string, string>) {
  const recipe = theme.value.recipes.tagInput;
  return recipe ? resolveRecipe(recipe, { variant, ...extra }) : [];
}

const rootClasses = computed(() =>
  cls("root", { size: props.size, disabled: props.disabled }),
);
const chipClasses = computed(() => cls("chip", { size: props.size }));
const deleteClasses = computed(() => cls("delete"));
const inputClasses = computed(() => cls("input", { size: props.size }));
const contentClasses = computed(() => cls("content"));
const itemClasses = computed(() => cls("item"));
const emptyClasses = computed(() => cls("empty"));
</script>

<template>
  <ComboboxRoot
    v-model:search-term="searchTerm"
    :model-value="model"
    :multiple="true"
    :open-on-focus="true"
    :ignore-filter="true"
    :reset-search-term-on-select="true"
    :disabled
    @update:model-value="(val: string[]) => (model = val)"
  >
    <TagsInputRoot
      v-model="model"
      :class="rootClasses"
      :disabled
      :delimiter="','"
      :add-on-blur="true"
      v-bind="$attrs"
    >
      <TagsInputItem
        v-for="tag in model"
        :key="tag"
        :value="tag"
        :class="chipClasses"
      >
        <TagsInputItemText />
        <TagsInputItemDelete :class="deleteClasses" aria-label="Remove">
          <AppIcon icon="lucide:x" size="sm" />
        </TagsInputItemDelete>
      </TagsInputItem>
      <ComboboxInput as-child>
        <TagsInputInput
          :placeholder="model.length === 0 ? placeholder : ''"
          :class="inputClasses"
        />
      </ComboboxInput>
    </TagsInputRoot>

    <ComboboxPortal>
      <ComboboxContent
        :class="contentClasses"
        position="popper"
        :side-offset="4"
      >
        <ComboboxViewport class="p-1 max-h-48 overflow-auto">
          <!-- Existing suggestions -->
          <ComboboxItem
            v-for="suggestion in filteredSuggestions"
            :key="suggestion"
            :value="suggestion"
            :class="itemClasses"
          >
            <ComboboxItemIndicator
              class="absolute left-2 inline-flex items-center"
            >
              <AppIcon icon="lucide:check" size="sm" />
            </ComboboxItemIndicator>
            <span class="pl-6">{{ suggestion }}</span>
          </ComboboxItem>

          <!-- Create new tag option -->
          <ComboboxItem
            v-if="showCreateOption"
            :value="searchTerm.trim()"
            :class="itemClasses"
            @select="handleSelect(searchTerm.trim())"
          >
            <span class="pl-6 flex items-center gap-1 text-primary">
              <AppIcon icon="lucide:plus" size="sm" />
              Create "{{ searchTerm.trim() }}"
            </span>
          </ComboboxItem>

          <!-- Empty state (no suggestions, no create) -->
          <ComboboxEmpty
            v-if="!filteredSuggestions.length && !showCreateOption"
            :class="emptyClasses"
          >
            Type to add a tag
          </ComboboxEmpty>
        </ComboboxViewport>
      </ComboboxContent>
    </ComboboxPortal>
  </ComboboxRoot>
</template>
