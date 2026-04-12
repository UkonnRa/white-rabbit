import { describe, it, expect } from "vitest";
import { mountWithTheme } from "../test-utils-mount";
import JournalCard from "./JournalCard.vue";
import type { Journal } from "../models";

const journal: Journal = {
  id: "1",
  name: "Personal Finance",
  description: "My daily expense tracking",
  tags: ["personal", "daily"],
  created_at: "2026-01-15T00:00:00Z",
  last_modified_at: null,
  archived_at: null,
};

const emptyJournal: Journal = {
  id: "2",
  name: "Empty",
  description: "",
  tags: [],
  created_at: null,
  last_modified_at: null,
  archived_at: null,
};

describe("JournalCard", () => {
  it("renders journal name", () => {
    const wrapper = mountWithTheme(JournalCard, {
      props: { journal },
    });
    expect(wrapper.text()).toContain("Personal Finance");
  });

  it("renders journal description", () => {
    const wrapper = mountWithTheme(JournalCard, {
      props: { journal },
    });
    expect(wrapper.text()).toContain("My daily expense tracking");
  });

  it("renders 'No description' placeholder when description is empty", () => {
    const wrapper = mountWithTheme(JournalCard, {
      props: { journal: emptyJournal },
    });
    expect(wrapper.text()).toContain("No description");
  });

  it("renders tags as chips", () => {
    const wrapper = mountWithTheme(JournalCard, {
      props: { journal },
    });
    expect(wrapper.text()).toContain("personal");
    expect(wrapper.text()).toContain("daily");
  });

  it("does not render tags section when tags are empty", () => {
    const wrapper = mountWithTheme(JournalCard, {
      props: { journal: emptyJournal },
    });
    const chips = wrapper.findAllComponents({ name: "AppChip" });
    expect(chips.length).toBe(0);
  });

  it("renders inside an AppCard component", () => {
    const wrapper = mountWithTheme(JournalCard, {
      props: { journal },
    });
    const card = wrapper.findComponent({ name: "AppCard" });
    expect(card.exists()).toBe(true);
  });

  it("emits click when card is clicked", async () => {
    const wrapper = mountWithTheme(JournalCard, {
      props: { journal },
    });
    await wrapper.findComponent({ name: "AppCard" }).trigger("click");
    expect(wrapper.emitted("click")).toBeTruthy();
  });

  it("emits edit when edit button is clicked (not click)", async () => {
    const wrapper = mountWithTheme(JournalCard, {
      props: { journal },
    });
    const editBtn = wrapper
      .findAllComponents({ name: "AppButton" })
      .find((b) => b.attributes("aria-label") === "Edit journal");
    expect(editBtn).toBeDefined();
    await editBtn!.trigger("click");
    expect(wrapper.emitted("edit")).toBeTruthy();
    // click should NOT have propagated
    expect(wrapper.emitted("click")).toBeFalsy();
  });

  it("emits delete when delete button is clicked (not click)", async () => {
    const wrapper = mountWithTheme(JournalCard, {
      props: { journal },
    });
    const deleteBtn = wrapper
      .findAllComponents({ name: "AppButton" })
      .find((b) => b.attributes("aria-label") === "Delete journal");
    expect(deleteBtn).toBeDefined();
    await deleteBtn!.trigger("click");
    expect(wrapper.emitted("delete")).toBeTruthy();
    expect(wrapper.emitted("click")).toBeFalsy();
  });

  it("renders created date when no last_modified_at", () => {
    const wrapper = mountWithTheme(JournalCard, {
      props: { journal },
    });
    expect(wrapper.text()).toMatch(/Created/);
  });

  it("renders updated date when last_modified_at exists", () => {
    const updatedJournal: Journal = {
      ...journal,
      last_modified_at: "2026-04-10T00:00:00Z",
    };
    const wrapper = mountWithTheme(JournalCard, {
      props: { journal: updatedJournal },
    });
    expect(wrapper.text()).toMatch(/Updated/);
  });

  it("has edit and delete action buttons", () => {
    const wrapper = mountWithTheme(JournalCard, {
      props: { journal },
    });
    const buttons = wrapper.findAllComponents({ name: "AppButton" });
    const editBtn = buttons.find(
      (b) => b.attributes("aria-label") === "Edit journal",
    );
    const deleteBtn = buttons.find(
      (b) => b.attributes("aria-label") === "Delete journal",
    );
    expect(editBtn).toBeDefined();
    expect(deleteBtn).toBeDefined();
  });
});
