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
    const buttons = wrapper.findAllComponents({ name: "AppButton" });
    const cancelBtn = buttons.find((b) => b.text().includes("✕"));
    expect(cancelBtn).toBeDefined();
    await cancelBtn!.trigger("click");
    expect(wrapper.emitted("cancelNew")).toBeTruthy();
  });

  it("renders edit and delete action buttons per row", () => {
    const wrapper = mountWithTheme(JournalTable, {
      props: { journals, addingNew: false },
    });
    const buttons = wrapper.findAllComponents({ name: "AppButton" });
    const editButtons = buttons.filter((b) => b.text().includes("✎"));
    const deleteButtons = buttons.filter((b) => b.text().includes("🗑"));
    expect(editButtons.length).toBe(3);
    expect(deleteButtons.length).toBe(3);
  });

  it("emits delete when delete button is clicked", async () => {
    const wrapper = mountWithTheme(JournalTable, {
      props: { journals, addingNew: false },
    });
    const deleteBtn = wrapper
      .findAllComponents({ name: "AppButton" })
      .find((b) => b.text().includes("🗑"));
    await deleteBtn!.trigger("click");
    expect(wrapper.emitted("delete")).toBeTruthy();
  });

  it("enters inline edit mode when edit button is clicked", async () => {
    const wrapper = mountWithTheme(JournalTable, {
      props: { journals, addingNew: false },
    });
    const editBtn = wrapper
      .findAllComponents({ name: "AppButton" })
      .find((b) => b.text().includes("✎"));
    await editBtn!.trigger("click");
    await nextTick();

    // Should now show ✓/✕ buttons instead of ✎/🗑
    const confirmBtn = wrapper
      .findAllComponents({ name: "AppButton" })
      .find((b) => b.text().includes("✓"));
    expect(confirmBtn).toBeDefined();
  });

  it("emits update when saving inline edit", async () => {
    const wrapper = mountWithTheme(JournalTable, {
      props: { journals, addingNew: false },
    });

    const editBtn = wrapper
      .findAllComponents({ name: "AppButton" })
      .find((b) => b.text().includes("✎"));
    await editBtn!.trigger("click");
    await nextTick();

    const saveBtn = wrapper
      .findAllComponents({ name: "AppButton" })
      .find((b) => b.text().includes("✓"));
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
      .find((b) => b.text().includes("✎"));
    await editBtn!.trigger("click");
    await nextTick();

    const cancelBtn = wrapper
      .findAllComponents({ name: "AppButton" })
      .find((b) => b.text().includes("✕"));
    await cancelBtn!.trigger("click");
    await nextTick();

    const editBtnAgain = wrapper
      .findAllComponents({ name: "AppButton" })
      .find((b) => b.text().includes("✎"));
    expect(editBtnAgain).toBeDefined();
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
});
