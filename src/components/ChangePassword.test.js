import { describe, expect, it } from "vitest";
import { mount } from "@vue/test-utils";
import ChangePassword from "./ChangePassword.vue";

const submitWith = async (wrapper, values) => {
  const inputs = wrapper.findAll("input");
  for (const [i, v] of values.entries()) await inputs[i].setValue(v);
  await wrapper.find("form").trigger("submit");
};

describe("ChangePassword", () => {
  it("emits current and new password when valid", async () => {
    // Arrange
    const wrapper = mount(ChangePassword);

    // Act
    await submitWith(wrapper, ["master-password-1", "nuova-password-12", "nuova-password-12"]);

    // Assert
    expect(wrapper.emitted("submit")[0]).toEqual([{ current: "master-password-1", next: "nuova-password-12" }]);
  });

  it("requires the current password", async () => {
    // Arrange
    const wrapper = mount(ChangePassword);

    // Act
    await submitWith(wrapper, ["", "nuova-password-12", "nuova-password-12"]);

    // Assert
    expect(wrapper.emitted("submit")).toBeUndefined();
    expect(wrapper.find(".error").text()).toContain("attuale");
  });

  it("rejects a too short new password", async () => {
    // Arrange
    const wrapper = mount(ChangePassword);

    // Act
    await submitWith(wrapper, ["master-password-1", "corta", "corta"]);

    // Assert
    expect(wrapper.emitted("submit")).toBeUndefined();
    expect(wrapper.find(".error").text()).toContain("almeno 12");
  });

  it("rejects mismatching confirmation", async () => {
    // Arrange
    const wrapper = mount(ChangePassword);

    // Act
    await submitWith(wrapper, ["master-password-1", "nuova-password-12", "nuova-password-13"]);

    // Assert
    expect(wrapper.emitted("submit")).toBeUndefined();
    expect(wrapper.find(".error").text()).toContain("non coincidono");
  });

  it("emits cancel from the cancel button", async () => {
    // Arrange
    const wrapper = mount(ChangePassword);

    // Act
    await wrapper.find("button[type=button]").trigger("click");

    // Assert
    expect(wrapper.emitted("cancel")).toHaveLength(1);
  });

  it("shows the backend error passed as prop", () => {
    // Arrange / Act
    const wrapper = mount(ChangePassword, { props: { error: "La password attuale non è corretta" } });

    // Assert
    expect(wrapper.find(".error").text()).toBe("La password attuale non è corretta");
  });
});
