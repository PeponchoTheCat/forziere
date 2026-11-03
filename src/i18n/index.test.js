import { describe, expect, it, vi } from "vitest";
import { DEFAULT_LOCALE, getInitialLocale, i18n, setLocale } from "./index";

describe("i18n", () => {
  it("defaults to English when nothing is saved", () => {
    // Arrange
    localStorage.clear();

    // Act
    const locale = getInitialLocale();

    // Assert
    expect(DEFAULT_LOCALE).toBe("en");
    expect(locale).toBe("en");
  });

  it("returns the saved language", () => {
    // Arrange
    localStorage.setItem("forziere.locale", "it");

    // Act
    const locale = getInitialLocale();

    // Assert
    expect(locale).toBe("it");
  });

  it("ignores an unsupported saved language", () => {
    // Arrange
    localStorage.setItem("forziere.locale", "fr");

    // Act
    const locale = getInitialLocale();

    // Assert
    expect(locale).toBe("en");
  });

  it("falls back to English when localStorage is not readable", () => {
    // Arrange
    const spy = vi.spyOn(Storage.prototype, "getItem").mockImplementation(() => {
      throw new Error("blocked");
    });

    // Act
    const locale = getInitialLocale();
    spy.mockRestore();

    // Assert
    expect(locale).toBe("en");
  });

  it("setLocale switches language, saves it and updates the page lang", () => {
    // Arrange / Act
    setLocale("it");

    // Assert
    expect(i18n.global.locale.value).toBe("it");
    expect(localStorage.getItem("forziere.locale")).toBe("it");
    expect(document.documentElement.lang).toBe("it");
  });

  it("setLocale ignores an unsupported language", () => {
    // Arrange
    setLocale("en");

    // Act
    setLocale("fr");

    // Assert
    expect(i18n.global.locale.value).toBe("en");
  });

  it("setLocale still switches when localStorage is not writable", () => {
    // Arrange
    const spy = vi.spyOn(Storage.prototype, "setItem").mockImplementation(() => {
      throw new Error("blocked");
    });

    // Act
    setLocale("it");
    spy.mockRestore();

    // Assert
    expect(i18n.global.locale.value).toBe("it");
  });
});
