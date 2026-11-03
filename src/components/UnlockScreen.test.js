import { describe, expect, it } from "vitest";
import { mount } from "@vue/test-utils";
import UnlockScreen from "./UnlockScreen.vue";

const submitWith = async (wrapper, values) => {
  const inputs = wrapper.findAll("input");
  for (const [i, v] of values.entries()) await inputs[i].setValue(v);
  await wrapper.find("form").trigger("submit");
};

describe("UnlockScreen", () => {
  it("unlock mode shows a single password field", () => {
    // Arrange / Act
    const wrapper = mount(UnlockScreen, { props: { mode: "unlock" } });

    // Assert
    expect(wrapper.findAll("input")).toHaveLength(1);
    expect(wrapper.find("button").text()).toBe("Unlock");
  });

  it("setup mode asks for confirmation and warns there is no recovery", () => {
    // Arrange / Act
    const wrapper = mount(UnlockScreen, { props: { mode: "setup" } });

    // Assert
    expect(wrapper.findAll("input")).toHaveLength(2);
    expect(wrapper.text()).toContain("There is no recovery");
  });

  it("migrate mode explains that the plaintext file will be encrypted", () => {
    // Arrange / Act
    const wrapper = mount(UnlockScreen, { props: { mode: "migrate" } });

    // Assert
    expect(wrapper.text()).toContain("in plaintext");
  });

  it("emits the password on unlock", async () => {
    // Arrange
    const wrapper = mount(UnlockScreen, { props: { mode: "unlock" } });

    // Act
    await submitWith(wrapper, ["master-password-1"]);

    // Assert
    expect(wrapper.emitted("submit")[0]).toEqual(["master-password-1"]);
  });

  it("does not emit on unlock when the field is empty", async () => {
    // Arrange
    const wrapper = mount(UnlockScreen, { props: { mode: "unlock" } });

    // Act
    await submitWith(wrapper, [""]);

    // Assert
    expect(wrapper.emitted("submit")).toBeUndefined();
    expect(wrapper.find(".error").text()).toBe("Enter the master password");
  });

  it("rejects a too short password in setup", async () => {
    // Arrange
    const wrapper = mount(UnlockScreen, { props: { mode: "setup" } });

    // Act
    await submitWith(wrapper, ["corta", "corta"]);

    // Assert
    expect(wrapper.emitted("submit")).toBeUndefined();
    expect(wrapper.find(".error").text()).toContain("at least 12");
  });

  it("rejects mismatching confirmation in setup", async () => {
    // Arrange
    const wrapper = mount(UnlockScreen, { props: { mode: "setup" } });

    // Act
    await submitWith(wrapper, ["master-password-1", "master-password-2"]);

    // Assert
    expect(wrapper.emitted("submit")).toBeUndefined();
    expect(wrapper.find(".error").text()).toContain("do not match");
  });

  it("emits in setup when password is long enough and confirmed", async () => {
    // Arrange
    const wrapper = mount(UnlockScreen, { props: { mode: "setup" } });

    // Act
    await submitWith(wrapper, ["master-password-1", "master-password-1"]);

    // Assert
    expect(wrapper.emitted("submit")[0]).toEqual(["master-password-1"]);
  });

  it("shows the backend error passed as prop", () => {
    // Arrange / Act
    const wrapper = mount(UnlockScreen, { props: { mode: "unlock", error: "Password errata" } });

    // Assert
    expect(wrapper.find(".error").text()).toBe("Password errata");
  });

  it("disables the button while busy", () => {
    // Arrange / Act
    const wrapper = mount(UnlockScreen, { props: { mode: "unlock", busy: true } });

    // Assert
    expect(wrapper.find("button").attributes("disabled")).toBeDefined();
  });
});
