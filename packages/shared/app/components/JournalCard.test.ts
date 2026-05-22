import { describe, it, expect } from "vitest";
import { mount } from "@vue/test-utils";
import { UCard, UButton, UBadge, UIcon } from "@nuxt/ui";
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
    const wrapper = mount(JournalCard, {
      props: { journal },
      global: { components: { UCard, UButton, UBadge, UIcon } },
    });
    expect(wrapper.text()).toContain("Personal Finance");
  });

  it("renders journal description", () => {
    const wrapper = mount(JournalCard, {
      props: { journal },
      global: { components: { UCard, UButton, UBadge, UIcon } },
    });
    expect(wrapper.text()).toContain("My daily expense tracking");
  });

  it("renders 'No description' placeholder when description is empty", () => {
    const wrapper = mount(JournalCard, {
      props: { journal: emptyJournal },
      global: { components: { UCard, UButton, UBadge, UIcon } },
    });
    expect(wrapper.text()).toContain("No description");
  });

  it("renders tags as badges", () => {
    const wrapper = mount(JournalCard, {
      props: { journal },
      global: { components: { UCard, UButton, UBadge, UIcon } },
    });
    expect(wrapper.text()).toContain("personal");
    expect(wrapper.text()).toContain("daily");
  });

  it("does not render tags section when tags are empty", () => {
    const wrapper = mount(JournalCard, {
      props: { journal: emptyJournal },
      global: { components: { UCard, UButton, UBadge, UIcon } },
    });
    expect(wrapper.text()).not.toContain("personal");
    expect(wrapper.text()).not.toContain("daily");
  });

  it("emits click when card is clicked", async () => {
    const wrapper = mount(JournalCard, {
      props: { journal },
      global: { components: { UCard, UButton, UBadge, UIcon } },
    });
    await wrapper.trigger("click");
    expect(wrapper.emitted("click")).toBeTruthy();
  });

  it("has edit and delete action buttons", () => {
    const wrapper = mount(JournalCard, {
      props: { journal },
      global: { components: { UCard, UButton, UBadge, UIcon } },
    });
    const html = wrapper.html();
    expect(html).toContain("lucide:pencil");
    expect(html).toContain("lucide:trash-2");
  });

  it("renders created date when no last_modified_at", () => {
    const wrapper = mount(JournalCard, {
      props: { journal },
      global: { components: { UCard, UButton, UBadge, UIcon } },
    });
    expect(wrapper.text()).toMatch(/Created/);
  });

  it("renders updated date when last_modified_at exists", () => {
    const updatedJournal: Journal = {
      ...journal,
      last_modified_at: "2026-04-10T00:00:00Z",
    };
    const wrapper = mount(JournalCard, {
      props: { journal: updatedJournal },
      global: { components: { UCard, UButton, UBadge, UIcon } },
    });
    expect(wrapper.text()).toMatch(/Updated/);
  });
});
