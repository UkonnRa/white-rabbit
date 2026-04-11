import { describe, it, expect } from "vitest";
import { mountWithTheme } from "../test-utils-mount";
import JournalCard from "./JournalCard.vue";
import type { Journal } from "../models";

const journal: Journal = {
  id: "1",
  name: "Test Journal",
  description: "A test journal description",
  tags: ["work", "daily"],
  created_at: "2026-01-01",
  last_modified_at: "2026-01-02",
  archived_at: null,
};

const journalNoTags: Journal = {
  ...journal,
  id: "2",
  description: "",
  tags: [],
};

describe("JournalCard", () => {
  it("renders journal name", () => {
    const wrapper = mountWithTheme(JournalCard, {
      props: { journal },
    });
    expect(wrapper.text()).toContain("Test Journal");
  });

  it("renders journal description", () => {
    const wrapper = mountWithTheme(JournalCard, {
      props: { journal },
    });
    expect(wrapper.text()).toContain("A test journal description");
  });

  it("does not render description when empty", () => {
    const wrapper = mountWithTheme(JournalCard, {
      props: { journal: journalNoTags },
    });
    // Should not have a description paragraph
    expect(wrapper.find("p").exists()).toBe(false);
  });

  it("renders tags as chips", () => {
    const wrapper = mountWithTheme(JournalCard, {
      props: { journal },
    });
    expect(wrapper.text()).toContain("work");
    expect(wrapper.text()).toContain("daily");
  });

  it("does not render tags section when empty", () => {
    const wrapper = mountWithTheme(JournalCard, {
      props: { journal: journalNoTags },
    });
    const chips = wrapper.findAllComponents({ name: "AppChip" });
    expect(chips).toHaveLength(0);
  });

  it("renders Edit and Delete buttons", () => {
    const wrapper = mountWithTheme(JournalCard, {
      props: { journal },
    });
    const buttons = wrapper.findAllComponents({ name: "AppButton" });
    expect(buttons.length).toBeGreaterThanOrEqual(2);
    expect(wrapper.text()).toContain("Edit");
    expect(wrapper.text()).toContain("Delete");
  });

  it("emits edit with journal on Edit click", async () => {
    const wrapper = mountWithTheme(JournalCard, {
      props: { journal },
    });
    const editBtn = wrapper
      .findAllComponents({ name: "AppButton" })
      .find((b) => b.text() === "Edit");
    expect(editBtn).toBeDefined();
    await editBtn!.trigger("click");
    expect(wrapper.emitted("edit")).toBeTruthy();
    expect(wrapper.emitted("edit")![0]).toEqual([journal]);
  });

  it("emits delete with journal on Delete click", async () => {
    const wrapper = mountWithTheme(JournalCard, {
      props: { journal },
    });
    const deleteBtn = wrapper
      .findAllComponents({ name: "AppButton" })
      .find((b) => b.text() === "Delete");
    expect(deleteBtn).toBeDefined();
    await deleteBtn!.trigger("click");
    expect(wrapper.emitted("delete")).toBeTruthy();
    expect(wrapper.emitted("delete")![0]).toEqual([journal]);
  });

  it("renders inside an AppCard component", () => {
    const wrapper = mountWithTheme(JournalCard, {
      props: { journal },
    });
    const card = wrapper.findComponent({ name: "AppCard" });
    expect(card.exists()).toBe(true);
  });
});
