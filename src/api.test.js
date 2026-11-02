import { beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));

import { invoke } from "@tauri-apps/api/core";
import { deleteEntry, generatePassword, listEntries, saveEntry } from "./api";

describe("api", () => {
  beforeEach(() => {
    invoke.mockReset();
  });

  it("listEntries invokes list_entries and returns its result", async () => {
    // Arrange
    const entries = [{ id: "1" }];
    invoke.mockResolvedValue(entries);

    // Act
    const result = await listEntries();

    // Assert
    expect(invoke).toHaveBeenCalledWith("list_entries");
    expect(result).toBe(entries);
  });

  it("saveEntry invokes save_entry wrapping the payload in `entry`", async () => {
    // Arrange
    const entry = { service: "GitHub" };
    invoke.mockResolvedValue({ id: "1", ...entry });

    // Act
    const result = await saveEntry(entry);

    // Assert
    expect(invoke).toHaveBeenCalledWith("save_entry", { entry });
    expect(result.id).toBe("1");
  });

  it("deleteEntry invokes delete_entry with the id", async () => {
    // Arrange
    invoke.mockResolvedValue(undefined);

    // Act
    await deleteEntry("abc");

    // Assert
    expect(invoke).toHaveBeenCalledWith("delete_entry", { id: "abc" });
  });

  it("generatePassword invokes generate_password wrapping options", async () => {
    // Arrange
    const options = { length: 12, uppercase: true, lowercase: true, digits: true, symbols: false };
    invoke.mockResolvedValue("secret");

    // Act
    const result = await generatePassword(options);

    // Assert
    expect(invoke).toHaveBeenCalledWith("generate_password", { options });
    expect(result).toBe("secret");
  });

  it("propagates backend errors to the caller", async () => {
    // Arrange
    invoke.mockRejectedValue("boom");

    // Act
    const promise = listEntries();

    // Assert
    await expect(promise).rejects.toBe("boom");
  });
});
