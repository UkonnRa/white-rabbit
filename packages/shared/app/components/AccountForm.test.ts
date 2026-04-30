import { describe, it, expect } from "vitest";
import { mountWithTheme } from "../test-utils-mount";
import AccountForm from "./AccountForm.vue";

describe("AccountForm", () => {
  it("renders name and parent path", () => {
    const wrapper = mountWithTheme(AccountForm, {
      props: { parentPath: "Asset > Bank" },
    });
    expect(wrapper.text()).toContain("Asset > Bank");
  });

  it("renders Name label and input", () => {
    const wrapper = mountWithTheme(AccountForm, {
      props: { parentPath: "" },
    });
    expect(wrapper.text()).toContain("Name");
    const nameInput = wrapper.find("#account-name");
    expect(nameInput.exists()).toBe(true);
  });

  it("renders Cancel and Create buttons for new form", () => {
    const wrapper = mountWithTheme(AccountForm, {
      props: { parentPath: "" },
    });
    expect(wrapper.text()).toContain("Cancel");
    expect(wrapper.text()).toContain("Create");
  });

  it("renders Cancel and Update buttons for edit form", () => {
    const wrapper = mountWithTheme(AccountForm, {
      props: {
        parentPath: "",
        initial: { name: "Test", description: "", tags: [] },
      },
    });
    expect(wrapper.text()).toContain("Cancel");
    expect(wrapper.text()).toContain("Update");
  });

  it("emits submit with form data", async () => {
    const wrapper = mountWithTheme(AccountForm, {
      props: { parentPath: "" },
    });

    const nameInput = wrapper.find("#account-name");
    await nameInput.setValue("Checking");

    const form = wrapper.find("form");
    await form.trigger("submit.prevent");

    expect(wrapper.emitted("submit")).toBeTruthy();
    const submitData = wrapper.emitted("submit")![0]![0] as {
      name: string;
      description: string;
      tags: string[];
    };
    expect(submitData.name).toBe("Checking");
    expect(submitData.description).toBe("");
    expect(submitData.tags).toEqual([]);
  });

  it("emits cancel when cancel button is clicked", async () => {
    const wrapper = mountWithTheme(AccountForm, {
      props: { parentPath: "" },
    });
    const buttons = wrapper.findAllComponents({ name: "AppButton" });
    const cancelBtn = buttons.find((b) => b.text() === "Cancel");
    expect(cancelBtn).toBeTruthy();
    if (cancelBtn) {
      await cancelBtn.trigger("click");
      expect(wrapper.emitted("cancel")).toBeTruthy();
    }
  });

  it("shows validation error for empty name", async () => {
    const wrapper = mountWithTheme(AccountForm, {
      props: { parentPath: "" },
    });

    const form = wrapper.find("form");
    await form.trigger("submit.prevent");

    expect(wrapper.text()).toContain("Name is required.");
    expect(wrapper.emitted("submit")).toBeFalsy();
  });

  it("shows validation error for reserved name", async () => {
    const wrapper = mountWithTheme(AccountForm, {
      props: { parentPath: "" },
    });

    const nameInput = wrapper.find("#account-name");
    await nameInput.setValue("Asset");

    const form = wrapper.find("form");
    await form.trigger("submit.prevent");

    expect(wrapper.text()).toContain("reserved root account name");
    expect(wrapper.emitted("submit")).toBeFalsy();
  });
});
