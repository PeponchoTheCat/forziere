import { describe, expect, it } from "vitest";
import { mount } from "@vue/test-utils";
import { i18n } from "../i18n";
import LanguageSwitch from "./LanguageSwitch.vue";

describe("LanguageSwitch", () => {
  it("lists English and Italian with the current language selected", () => {
    // Arrange / Act
    const wrapper = mount(LanguageSwitch);

    // Assert
    expect(wrapper.findAll("option").map((o) => o.text())).toEqual(["English", "Italiano"]);
    expect(wrapper.find("select").element.value).toBe("en");
  });

  it("changes the language and remembers the choice", async () => {
    // Arrange
    const wrapper = mount(LanguageSwitch);

    // Act
    await wrapper.find("select").setValue("it");

    // Assert
    expect(i18n.global.locale.value).toBe("it");
    expect(localStorage.getItem("forziere.locale")).toBe("it");
    expect(wrapper.find("select").attributes("aria-label")).toBe("Lingua");
  });
});
