<script setup lang="ts" generic="TData">
import {
  FlexRender,
  getCoreRowModel,
  getSortedRowModel,
  getFilteredRowModel,
  getExpandedRowModel,
  useVueTable,
} from "@tanstack/vue-table";
import type {
  ColumnDef,
  SortingState,
  ColumnFiltersState,
  ExpandedState,
} from "@tanstack/vue-table";
import { ref, computed } from "vue";
import { resolveRecipe } from "../../composables/useRecipe";
import { useTheme } from "../../composables/useTheme";

const props = defineProps<{
  data: TData[];
  columns: ColumnDef<TData, unknown>[];
  getSubRows?: (row: TData) => TData[];
  enableExpanding?: boolean;
  expanded?: ExpandedState;
}>();

const emit = defineEmits<{
  (e: "update:expanded", value: ExpandedState): void;
}>();

const sorting = ref<SortingState>([]);
const columnFilters = ref<ColumnFiltersState>([]);
const internalExpanded = ref<ExpandedState>({});

const expandedState = computed(() =>
  props.expanded !== undefined ? props.expanded : internalExpanded.value,
);

function handleExpandedChange(updater: unknown) {
  const next =
    typeof updater === "function" ? updater(expandedState.value) : updater;
  if (props.expanded !== undefined) {
    emit("update:expanded", next);
  } else {
    internalExpanded.value = next;
  }
}

const table = useVueTable({
  get data() {
    return props.data;
  },
  get columns() {
    return props.columns;
  },
  state: {
    get sorting() {
      return sorting.value;
    },
    get columnFilters() {
      return columnFilters.value;
    },
    ...(props.enableExpanding
      ? {
          get expanded() {
            return expandedState.value;
          },
        }
      : {}),
  },
  onSortingChange: (updater) => {
    sorting.value =
      typeof updater === "function" ? updater(sorting.value) : updater;
  },
  onColumnFiltersChange: (updater) => {
    columnFilters.value =
      typeof updater === "function" ? updater(columnFilters.value) : updater;
  },
  ...(props.enableExpanding
    ? {
        onExpandedChange: (updater: unknown) => {
          handleExpandedChange(updater);
        },
      }
    : {}),
  getSubRows: props.getSubRows,
  getCoreRowModel: getCoreRowModel(),
  getSortedRowModel: getSortedRowModel(),
  getFilteredRowModel: getFilteredRowModel(),
  getExpandedRowModel: props.enableExpanding
    ? getExpandedRowModel()
    : undefined,
});

const { theme } = useTheme();

function cls(variant: string) {
  const recipe = theme.value.recipes.dataTable;
  return recipe ? resolveRecipe(recipe, { variant }) : [];
}

const tableClasses = computed(() => cls("table"));
const headerClasses = computed(() => cls("header"));
const headerCellClasses = computed(() => cls("headerCell"));
const rowClasses = computed(() => cls("row"));
const cellClasses = computed(() => cls("cell"));
const emptyClasses = computed(() => cls("empty"));
</script>

<template>
  <div class="w-full overflow-auto">
    <table :class="tableClasses">
      <thead :class="headerClasses">
        <tr
          v-for="headerGroup in table.getHeaderGroups()"
          :key="headerGroup.id"
        >
          <th
            v-for="header in headerGroup.headers"
            :key="header.id"
            :class="headerCellClasses"
            :style="{
              width:
                header.getSize() !== 150 ? `${header.getSize()}px` : undefined,
            }"
          >
            <slot
              :name="`header-${header.column.id}`"
              :header="header"
              :table="table"
            >
              <div
                v-if="!header.isPlaceholder"
                :class="{
                  'cursor-pointer select-none': header.column.getCanSort(),
                }"
                @click="header.column.getToggleSortingHandler()?.($event)"
              >
                <FlexRender
                  :render="header.column.columnDef.header"
                  :props="header.getContext()"
                />
                <span v-if="header.column.getIsSorted() === 'asc'"> ↑</span>
                <span v-else-if="header.column.getIsSorted() === 'desc'">
                  ↓</span
                >
              </div>
            </slot>
          </th>
        </tr>
      </thead>
      <tbody>
        <slot name="body-prepend" :table="table" />
        <template v-if="table.getRowModel().rows.length">
          <tr
            v-for="row in table.getRowModel().rows"
            :key="row.id"
            :class="rowClasses"
          >
            <td
              v-for="cell in row.getVisibleCells()"
              :key="cell.id"
              :class="cellClasses"
            >
              <slot :name="`cell-${cell.column.id}`" :cell="cell" :row="row">
                <FlexRender
                  :render="cell.column.columnDef.cell"
                  :props="cell.getContext()"
                />
              </slot>
            </td>
          </tr>
        </template>
        <tr v-else>
          <td :colspan="table.getAllColumns().length" :class="emptyClasses">
            <slot name="empty">No data.</slot>
          </td>
        </tr>
      </tbody>
    </table>
  </div>
</template>
