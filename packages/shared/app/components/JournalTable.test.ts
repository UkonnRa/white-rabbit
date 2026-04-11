import { describe, it, expect } from "vitest";
import { nextTick } from "vue";
import { mountWithTheme } from "../test-utils-mount";
import JournalTable from "./JournalTable.vue";
import type { Journal } from "../models";

const journals: Journal[] = [
  {
    id: "1",
    name: "Alpha",
    description: "First journal",
    tags: ["work"],
    created_at: "2026-01-01",
    last_modified_at: null,
    archived_at: null,
  },
  {
    id: "2",
    name: "Beta",
    description: "Second journal",
    tags: ["personal", "daily"],
    created_at: "2026-01-02",
    last_modified_at: null,
    archived_at: null,
  },
  {
    id: "3",
    name: "Gamma",
    description: "",
    tags: [],
    created_at: "2026-01-03",
    last_modified_at: null,
    archived_at: null,
  },
];

describe("JournalTable", () => {
  it("renders all journal rows", () => {
    const wrapper = mountWithTheme(JournalTable, {
      props: { journals, addingNew: false },
    });
    expect(wrapper.text()).toContain("Alpha");
    expect(wrapper.text()).toContain("Beta");
    expect(wrapper.text()).toContain("Gamma");
  });

  it("renders journal descriptions", () => {
    const wrapper = mountWithTheme(JournalTable, {
      props: { journals, addingNew: false },
    });
    expect(wrapper.text()).toContain("First journal");
    expect(wrapper.text()).toContain("Second journal");
  });

  it("renders em dash for empty description", () => {
    const wrapper = mountWithTheme(JournalTable, {
      props: { journals, addingNew: false },
    });
    const cells = wrapper.findAll("td");
    const dashCell = cells.find((c) => c.text() === "—");
    expect(dashCell).toBeDefined();
  });

  it("renders tags as chips", () => {
    const wrapper = mountWithTheme(JournalTable, {
      props: { journals, addingNew: false },
    });
    expect(wrapper.text()).toContain("work");
    expect(wrapper.text()).toContain("personal");
    expect(wrapper.text()).toContain("daily");
  });

  it("renders filter inputs in header", () => {
    const wrapper = mountWithTheme(JournalTable, {
      props: { journals, addingNew: false },
    });
    expect(wrapper.find('input[placeholder="Filter name…"]').exists()).toBe(
      true,
    );
    expect(
      wrapper.find('input[placeholder="Filter description…"]').exists(),
    ).toBe(true);
    expect(wrapper.find('input[placeholder="Filter tags…"]').exists()).toBe(
      true,
    );
  });

  it("shows new row when addingNew is true", () => {
    const wrapper = mountWithTheme(JournalTable, {
      props: { journals, addingNew: true },
    });
    expect(wrapper.find('input[placeholder="Journal name"]').exists()).toBe(
      true,
    );
  });

  it("does not show new row when addingNew is false", () => {
    const wrapper = mountWithTheme(JournalTable, {
      props: { journals, addingNew: false },
    });
    expect(wrapper.find('input[placeholder="Journal name"]').exists()).toBe(
      false,
    );
  });

  it("emits cancelNew when cancel button in new row is clicked", async () => {
    const wrapper = mountWithTheme(JournalTable, {
      props: { journals, addingNew: true },
    });
    // In the new row, the cancel button is outlined+sm (second button after the solid save button)
    const buttons = wrapper.findAllComponents({ name: "AppButton" });
    const cancelBtn = buttons.find(
      (b) => b.props("variant") === "outlined" && b.props("size") === "sm",
    );
    expect(cancelBtn).toBeDefined();
    await cancelBtn!.trigger("click");
    expect(wrapper.emitted("cancelNew")).toBeTruthy();
  });

  it("renders edit and delete action buttons per row", () => {
    const wrapper = mountWithTheme(JournalTable, {
      props: { journals, addingNew: false },
    });
    // Each row has 2 outlined action buttons (edit + delete) — 3 rows × 2 = 6
    const buttons = wrapper.findAllComponents({ name: "AppButton" });
    const outlinedButtons = buttons.filter(
      (b) => b.props("variant") === "outlined",
    );
    expect(outlinedButtons.length).toBe(6);
  });

  it("emits delete when delete button is clicked", async () => {
    const wrapper = mountWithTheme(JournalTable, {
      props: { journals, addingNew: false },
    });
    // Delete buttons have class text-error
    const deleteBtn = wrapper
      .findAllComponents({ name: "AppButton" })
      .find((b) => b.classes().includes("text-error"));
    await deleteBtn!.trigger("click");
    expect(wrapper.emitted("delete")).toBeTruthy();
  });

  it("enters inline edit mode when edit button is clicked", async () => {
    const wrapper = mountWithTheme(JournalTable, {
      props: { journals, addingNew: false },
    });
    // Edit buttons are outlined without text-error class
    const editBtn = wrapper
      .findAllComponents({ name: "AppButton" })
      .find(
        (b) =>
          b.props("variant") === "outlined" &&
          !b.classes().includes("text-error"),
      );
    await editBtn!.trigger("click");
    await nextTick();

    // Should now show a solid (save) button
    const confirmBtn = wrapper
      .findAllComponents({ name: "AppButton" })
      .find((b) => b.props("variant") === "solid" && b.props("size") === "sm");
    expect(confirmBtn).toBeDefined();
  });

  it("emits update when saving inline edit", async () => {
    const wrapper = mountWithTheme(JournalTable, {
      props: { journals, addingNew: false },
    });

    const editBtn = wrapper
      .findAllComponents({ name: "AppButton" })
      .find(
        (b) =>
          b.props("variant") === "outlined" &&
          !b.classes().includes("text-error"),
      );
    await editBtn!.trigger("click");
    await nextTick();

    const saveBtn = wrapper
      .findAllComponents({ name: "AppButton" })
      .find((b) => b.props("variant") === "solid" && b.props("size") === "sm");
    await saveBtn!.trigger("click");

    expect(wrapper.emitted("update")).toBeTruthy();
    const [id] = wrapper.emitted("update")![0] as [string, unknown];
    expect(id).toBe("1");
  });

  it("cancels inline edit when cancel button clicked", async () => {
    const wrapper = mountWithTheme(JournalTable, {
      props: { journals, addingNew: false },
    });

    const editBtn = wrapper
      .findAllComponents({ name: "AppButton" })
      .find(
        (b) =>
          b.props("variant") === "outlined" &&
          !b.classes().includes("text-error"),
      );
    await editBtn!.trigger("click");
    await nextTick();

    // Cancel in edit mode is the outlined sm button (not solid)
    const cancelBtn = wrapper
      .findAllComponents({ name: "AppButton" })
      .find(
        (b) =>
          b.props("variant") === "outlined" &&
          b.props("size") === "sm" &&
          !b.classes().includes("text-error"),
      );
    await cancelBtn!.trigger("click");
    await nextTick();

    // Should be back to normal mode — delete buttons visible again
    const deleteBtn = wrapper
      .findAllComponents({ name: "AppButton" })
      .find((b) => b.classes().includes("text-error"));
    expect(deleteBtn).toBeDefined();
  });

  it("renders inside an AppDataTable", () => {
    const wrapper = mountWithTheme(JournalTable, {
      props: { journals, addingNew: false },
    });
    const dataTable = wrapper.findComponent({ name: "AppDataTable" });
    expect(dataTable.exists()).toBe(true);
  });

  it("shows empty state with no journals", () => {
    const wrapper = mountWithTheme(JournalTable, {
      props: { journals: [], addingNew: false },
    });
    expect(wrapper.text()).toContain("No journals found.");
  });

  it("renders table headers", () => {
    const wrapper = mountWithTheme(JournalTable, {
      props: { journals, addingNew: false },
    });
    expect(wrapper.text()).toContain("Name");
    expect(wrapper.text()).toContain("Description");
    expect(wrapper.text()).toContain("Tags");
    expect(wrapper.text()).toContain("Actions");
  });

  // ── Filter tests ──────────────────────────────────────────────────────────

  it("filters rows by name", async () => {
    const wrapper = mountWithTheme(JournalTable, {
      props: { journals, addingNew: false },
    });
    const nameInput = wrapper.find('input[placeholder="Filter name…"]');
    await nameInput.setValue("Alpha");
    await nextTick();

    expect(wrapper.text()).toContain("Alpha");
    expect(wrapper.text()).not.toContain("Beta");
    expect(wrapper.text()).not.toContain("Gamma");
  });

  it("filters rows by description", async () => {
    const wrapper = mountWithTheme(JournalTable, {
      props: { journals, addingNew: false },
    });
    const descInput = wrapper.find('input[placeholder="Filter description…"]');
    await descInput.setValue("Second");
    await nextTick();

    expect(wrapper.text()).toContain("Beta");
    expect(wrapper.text()).not.toContain("Alpha");
    expect(wrapper.text()).not.toContain("Gamma");
  });

  it("filters rows by tag", async () => {
    const wrapper = mountWithTheme(JournalTable, {
      props: { journals, addingNew: false },
    });
    const tagsInput = wrapper.find('input[placeholder="Filter tags…"]');
    await tagsInput.setValue("personal");
    await nextTick();

    expect(wrapper.text()).toContain("Beta");
    expect(wrapper.text()).not.toContain("Alpha");
    expect(wrapper.text()).not.toContain("Gamma");
  });

  it("shows no rows when filter matches nothing", async () => {
    const wrapper = mountWithTheme(JournalTable, {
      props: { journals, addingNew: false },
    });
    const nameInput = wrapper.find('input[placeholder="Filter name…"]');
    await nameInput.setValue("nonexistent");
    await nextTick();

    expect(wrapper.text()).not.toContain("Alpha");
    expect(wrapper.text()).not.toContain("Beta");
    expect(wrapper.text()).not.toContain("Gamma");
    expect(wrapper.text()).toContain("No journals found.");
  });

  it("filter is case-insensitive", async () => {
    const wrapper = mountWithTheme(JournalTable, {
      props: { journals, addingNew: false },
    });
    const nameInput = wrapper.find('input[placeholder="Filter name…"]');
    await nameInput.setValue("alpha");
    await nextTick();

    expect(wrapper.text()).toContain("Alpha");
    expect(wrapper.text()).not.toContain("Beta");
  });

  it("clears filter to show all rows again", async () => {
    const wrapper = mountWithTheme(JournalTable, {
      props: { journals, addingNew: false },
    });
    const nameInput = wrapper.find('input[placeholder="Filter name…"]');
    await nameInput.setValue("Alpha");
    await nextTick();
    expect(wrapper.text()).not.toContain("Beta");

    await nameInput.setValue("");
    await nextTick();
    expect(wrapper.text()).toContain("Alpha");
    expect(wrapper.text()).toContain("Beta");
    expect(wrapper.text()).toContain("Gamma");
  });
});
