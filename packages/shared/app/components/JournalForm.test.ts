import { describe, it, expect } from "vitest";
import { nextTick } from "vue";
import { mountWithTheme } from "../test-utils-mount";
import JournalForm from "./JournalForm.vue";

describe("JournalForm", () => {
  it("renders 'New Journal' title when no initial", () => {
    const wrapper = mountWithTheme(JournalForm);
    expect(wrapper.text()).toContain("New Journal");
  });

  it("renders 'Edit Journal' title when initial is provided", () => {
    const wrapper = mountWithTheme(JournalForm, {
      props: {
        initial: { name: "My Journal", description: "Desc", tags: ["a"] },
      },
    });
    expect(wrapper.text()).toContain("Edit Journal");
  });

  it("renders name, description, and tags labels", () => {
    const wrapper = mountWithTheme(JournalForm);
    expect(wrapper.text()).toContain("Name");
    expect(wrapper.text()).toContain("Description");
    expect(wrapper.text()).toContain("Tags");
  });

  it("renders pre-existing tags from initial prop", () => {
    const wrapper = mountWithTheme(JournalForm, {
      props: {
        initial: {
          name: "Test",
          description: "A desc",
          tags: ["tag1", "tag2"],
        },
      },
    });
    expect(wrapper.text()).toContain("tag1");
    expect(wrapper.text()).toContain("tag2");
  });

  it("renders Cancel and Create buttons for new form", () => {
    const wrapper = mountWithTheme(JournalForm);
    expect(wrapper.text()).toContain("Cancel");
    expect(wrapper.text()).toContain("Create");
  });

  it("renders Cancel and Update buttons for edit form", () => {
    const wrapper = mountWithTheme(JournalForm, {
      props: {
        initial: { name: "Test", description: "", tags: [] },
      },
    });
    expect(wrapper.text()).toContain("Cancel");
    expect(wrapper.text()).toContain("Update");
  });

  it("emits cancel when Cancel button is clicked", async () => {
    const wrapper = mountWithTheme(JournalForm);
    const cancelBtn = wrapper
      .findAllComponents({ name: "AppButton" })
      .find((b) => b.text() === "Cancel");
    expect(cancelBtn).toBeDefined();
    await cancelBtn!.trigger("click");
    expect(wrapper.emitted("cancel")).toBeTruthy();
  });

  it("emits submit on form submit", async () => {
    const wrapper = mountWithTheme(JournalForm);
    await wrapper.find("form").trigger("submit");
    expect(wrapper.emitted("submit")).toBeTruthy();
  });

  it("removes a tag when x button is clicked", async () => {
    const wrapper = mountWithTheme(JournalForm, {
      props: {
        initial: { name: "Test", description: "", tags: ["removeme"] },
      },
    });

    expect(wrapper.text()).toContain("removeme");

    const removeBtn = wrapper.find('button[aria-label="Remove tag"]');
    expect(removeBtn.exists()).toBe(true);
    await removeBtn.trigger("click");
    await nextTick();

    expect(wrapper.text()).not.toContain("removeme");
  });

  it("renders inside an AppCard component", () => {
    const wrapper = mountWithTheme(JournalForm);
    const card = wrapper.findComponent({ name: "AppCard" });
    expect(card.exists()).toBe(true);
  });

  it("has required name input", () => {
    const wrapper = mountWithTheme(JournalForm);
    const nameInput = wrapper.find("#journal-name");
    expect(nameInput.exists()).toBe(true);
    expect(nameInput.attributes("placeholder")).toBe("Journal name");
  });

  it("has description textarea", () => {
    const wrapper = mountWithTheme(JournalForm);
    const textarea = wrapper.find("#journal-description");
    expect(textarea.exists()).toBe(true);
  });

  it("has tag input", () => {
    const wrapper = mountWithTheme(JournalForm);
    const tagInput = wrapper.find("#journal-tags");
    expect(tagInput.exists()).toBe(true);
  });
});
