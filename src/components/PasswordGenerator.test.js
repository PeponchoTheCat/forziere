import { beforeEach, describe, expect, it, vi } from "vitest";
import { flushPromises, mount } from "@vue/test-utils";

vi.mock("../api", () => ({ generatePassword: vi.fn() }));

import { generatePassword } from "../api";
import PasswordGenerator from "./PasswordGenerator.vue";

const checkbox = (wrapper, index) => wrapper.findAll('input[type="checkbox"]')[index];
const buttons = (wrapper) => wrapper.findAll("button");

describe("PasswordGenerator", () => {
  beforeEach(() => {
    generatePassword.mockReset();
  });

  it("starts with length 16 and all classes enabled", () => {
    // Arrange & Act
    const wrapper = mount(PasswordGenerator);

    // Assert
    expect(wrapper.find('input[type="range"]').element.value).toBe("16");
    for (let i = 0; i < 4; i++) expect(checkbox(wrapper, i).element.checked).toBe(true);
  });

  it("restricts the length slider between 4 and 128", () => {
    // Arrange & Act
    const wrapper = mount(PasswordGenerator);

    // Assert
    const range = wrapper.find('input[type="range"]');
    expect(range.attributes("min")).toBe("4");
    expect(range.attributes("max")).toBe("128");
  });

  it("displays the selected length", async () => {
    // Arrange
    const wrapper = mount(PasswordGenerator);

    // Act
    await wrapper.find('input[type="range"]').setValue("40");

    // Assert
    expect(wrapper.find("strong").text()).toBe("40");
  });

  it("calls generatePassword with the chosen options and numeric length", async () => {
    // Arrange
    generatePassword.mockResolvedValue("abc");
    const wrapper = mount(PasswordGenerator);
    await wrapper.find('input[type="range"]').setValue("24");
    await checkbox(wrapper, 0).setValue(false); // maiuscole
    await checkbox(wrapper, 3).setValue(false); // speciali

    // Act
    await buttons(wrapper)[0].trigger("click");
    await flushPromises();

    // Assert
    expect(generatePassword).toHaveBeenCalledWith({
      length: 24,
      uppercase: false,
      lowercase: true,
      digits: true,
      symbols: false,
    });
  });

  it("shows the generated password", async () => {
    // Arrange
    generatePassword.mockResolvedValue("S3cret!");
    const wrapper = mount(PasswordGenerator);

    // Act
    await buttons(wrapper)[0].trigger("click");
    await flushPromises();

    // Assert
    expect(wrapper.find("code.out").text()).toBe("S3cret!");
  });

  it("shows the backend error and hides the previous password on failure", async () => {
    // Arrange
    generatePassword.mockResolvedValueOnce("old");
    const wrapper = mount(PasswordGenerator);
    await buttons(wrapper)[0].trigger("click");
    await flushPromises();
    generatePassword.mockRejectedValueOnce("Seleziona almeno un tipo di carattere");

    // Act
    await buttons(wrapper)[0].trigger("click");
    await flushPromises();

    // Assert
    expect(wrapper.find(".error").text()).toBe("Seleziona almeno un tipo di carattere");
    expect(wrapper.find("code.out").exists()).toBe(false);
  });

  it("clears the previous error after a successful generation", async () => {
    // Arrange
    generatePassword.mockRejectedValueOnce("errore");
    const wrapper = mount(PasswordGenerator);
    await buttons(wrapper)[0].trigger("click");
    await flushPromises();
    generatePassword.mockResolvedValueOnce("ok");

    // Act
    await buttons(wrapper)[0].trigger("click");
    await flushPromises();

    // Assert
    expect(wrapper.find(".error").exists()).toBe(false);
    expect(wrapper.find("code.out").text()).toBe("ok");
  });

  it("disables the use button until a password is generated", async () => {
    // Arrange
    generatePassword.mockResolvedValue("abc");
    const wrapper = mount(PasswordGenerator);
    const before = buttons(wrapper)[1].attributes("disabled");

    // Act
    await buttons(wrapper)[0].trigger("click");
    await flushPromises();

    // Assert
    expect(before).toBeDefined();
    expect(buttons(wrapper)[1].attributes("disabled")).toBeUndefined();
  });

  it("emits use with the generated password", async () => {
    // Arrange
    generatePassword.mockResolvedValue("abc123");
    const wrapper = mount(PasswordGenerator);
    await buttons(wrapper)[0].trigger("click");
    await flushPromises();

    // Act
    await buttons(wrapper)[1].trigger("click");

    // Assert
    expect(wrapper.emitted("use")).toEqual([["abc123"]]);
  });
});
