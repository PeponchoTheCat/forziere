import { describe, expect, it } from "vitest";
import { setLocale } from "./index";
import { translateError } from "./errors";

describe("translateError", () => {
  it("translates every backend error code in both languages", () => {
    // Arrange
    const codes = [
      "vault_locked", "bad_password", "wrong_current_password", "password_too_short:12", "vault_exists",
      "already_unlocked", "no_vault_to_unlock", "entry_required_fields", "no_char_class", "invalid_length:4:128",
      "kdf_unsupported", "kdf_invalid_params", "kdf_failed", "encrypt_failed", "vault_version_unsupported",
      "vault_format_unknown",
    ];

    // Act
    const untranslated = [];
    for (const lang of ["en", "it"]) {
      setLocale(lang);
      for (const code of codes) if (translateError(code) === code) untranslated.push(`${lang}:${code}`);
    }

    // Assert
    expect(untranslated).toEqual([]);
  });

  it("translates a plain error code", () => {
    // Arrange / Act
    const text = translateError("vault_locked");

    // Assert
    expect(text).toBe("Vault locked");
  });

  it("translates the same code in Italian after a language change", () => {
    // Arrange
    setLocale("it");

    // Act
    const text = translateError("vault_locked");

    // Assert
    expect(text).toBe("Vault bloccato");
  });

  it("fills a single parameter", () => {
    // Arrange / Act
    const text = translateError("password_too_short:12");

    // Assert
    expect(text).toBe("The master password must be at least 12 characters long");
  });

  it("fills several parameters", () => {
    // Arrange
    setLocale("it");

    // Act
    const text = translateError("invalid_length:4:128");

    // Assert
    expect(text).toBe("La lunghezza deve essere tra 4 e 128");
  });

  it("passes an unknown message through unchanged", () => {
    // Arrange / Act
    const text = translateError("No such file or directory (os error 2)");

    // Assert
    expect(text).toBe("No such file or directory (os error 2)");
  });

  it("accepts non-string errors", () => {
    // Arrange / Act
    const text = translateError(new Error("boom"));

    // Assert
    expect(text).toBe("Error: boom");
  });
});
