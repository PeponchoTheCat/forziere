import { beforeEach, describe, expect, it, vi } from "vitest";
import { flushPromises, mount } from "@vue/test-utils";

vi.mock("./api", () => ({
  listEntries: vi.fn(),
  saveEntry: vi.fn(),
  deleteEntry: vi.fn(),
  generatePassword: vi.fn(),
  vaultStatus: vi.fn(),
  setupVault: vi.fn(),
  unlockVault: vi.fn(),
  lockVault: vi.fn(),
  changePassword: vi.fn(),
}));

import {
  changePassword,
  deleteEntry,
  listEntries,
  lockVault,
  saveEntry,
  setupVault,
  unlockVault,
  vaultStatus,
} from "./api";
import App from "./App.vue";

const entry = (id, service) => ({
  id,
  service,
  url: "",
  username: "user",
  password: "pw",
  notes: "",
  tags: [],
  created_at: "2026-10-23T00:00:00Z",
  updated_at: "2026-10-23T00:00:00Z",
});

const mountApp = async () => {
  const wrapper = mount(App);
  await flushPromises();
  return wrapper;
};

describe("App", () => {
  beforeEach(() => {
    listEntries.mockReset().mockResolvedValue([]);
    saveEntry.mockReset();
    deleteEntry.mockReset();
    vaultStatus.mockReset().mockResolvedValue("unlocked");
    setupVault.mockReset().mockResolvedValue(undefined);
    unlockVault.mockReset().mockResolvedValue(undefined);
    lockVault.mockReset().mockResolvedValue(undefined);
    changePassword.mockReset().mockResolvedValue(undefined);
    vi.spyOn(window, "confirm").mockReturnValue(true);
  });

  it("loads and lists entries on mount", async () => {
    // Arrange
    listEntries.mockResolvedValue([entry("1", "GitHub"), entry("2", "GitLab")]);

    // Act
    const wrapper = await mountApp();

    // Assert
    expect(wrapper.findAll("li")).toHaveLength(2);
  });

  it("shows the load error in the form when listing fails", async () => {
    // Arrange
    listEntries.mockRejectedValue("disco non leggibile");

    // Act
    const wrapper = await mountApp();

    // Assert
    expect(wrapper.find(".error").text()).toBe("disco non leggibile");
  });

  it("shows the new-entry form initially", async () => {
    // Arrange & Act
    const wrapper = await mountApp();

    // Assert
    expect(wrapper.find("h2").text()).toBe("New entry");
  });

  it("opens the edit form when an entry is selected", async () => {
    // Arrange
    listEntries.mockResolvedValue([entry("1", "GitHub")]);
    const wrapper = await mountApp();

    // Act
    await wrapper.find("li").trigger("click");

    // Assert
    expect(wrapper.find("h2").text()).toBe("Edit entry");
  });

  it("returns to an empty new-entry form when the new button is clicked", async () => {
    // Arrange
    listEntries.mockResolvedValue([entry("1", "GitHub")]);
    const wrapper = await mountApp();
    await wrapper.find("li").trigger("click");

    // Act
    await wrapper.find(".toolbar button").trigger("click");

    // Assert
    expect(wrapper.find("h2").text()).toBe("New entry");
  });

  it("saves, reloads the list and selects the saved entry", async () => {
    // Arrange
    const saved = entry("9", "Nuovo");
    saveEntry.mockResolvedValue(saved);
    const wrapper = await mountApp();
    listEntries.mockResolvedValue([saved]);
    const inputs = wrapper.findAll("form input");
    await inputs[0].setValue("Nuovo");
    await inputs[2].setValue("user");

    // Act
    await wrapper.find("form").trigger("submit");
    await flushPromises();

    // Assert
    expect(saveEntry).toHaveBeenCalledOnce();
    expect(saveEntry.mock.calls[0][0]).toMatchObject({ id: null, service: "Nuovo", username: "user" });
    expect(wrapper.findAll("li")).toHaveLength(1);
    expect(wrapper.find("li.active").exists()).toBe(true);
    expect(wrapper.find("h2").text()).toBe("Edit entry");
  });

  it("shows the backend error when saving fails and keeps the list unchanged", async () => {
    // Arrange
    saveEntry.mockRejectedValue("entry_required_fields");
    const wrapper = await mountApp();
    const inputs = wrapper.findAll("form input");
    await inputs[0].setValue("x");
    await inputs[2].setValue("y");

    // Act
    await wrapper.find("form").trigger("submit");
    await flushPromises();

    // Assert
    expect(wrapper.find(".error").text()).toBe("Service and username are required");
    expect(listEntries).toHaveBeenCalledTimes(1);
  });

  it("deletes the selected entry after confirmation and reloads", async () => {
    // Arrange
    listEntries.mockResolvedValue([entry("1", "GitHub")]);
    deleteEntry.mockResolvedValue(undefined);
    const wrapper = await mountApp();
    await wrapper.find("li").trigger("click");
    listEntries.mockResolvedValue([]);

    // Act
    await wrapper.find("button.danger").trigger("click");
    await flushPromises();

    // Assert
    expect(deleteEntry).toHaveBeenCalledWith("1");
    expect(wrapper.findAll("li")).toHaveLength(0);
    expect(wrapper.find("h2").text()).toBe("New entry");
  });

  it("does not delete when the confirmation is declined", async () => {
    // Arrange
    window.confirm.mockReturnValue(false);
    listEntries.mockResolvedValue([entry("1", "GitHub")]);
    const wrapper = await mountApp();
    await wrapper.find("li").trigger("click");

    // Act
    await wrapper.find("button.danger").trigger("click");
    await flushPromises();

    // Assert
    expect(deleteEntry).not.toHaveBeenCalled();
    expect(wrapper.findAll("li")).toHaveLength(1);
  });

  it("shows the backend error when deleting fails", async () => {
    // Arrange
    listEntries.mockResolvedValue([entry("1", "GitHub")]);
    deleteEntry.mockRejectedValue("errore di scrittura");
    const wrapper = await mountApp();
    await wrapper.find("li").trigger("click");

    // Act
    await wrapper.find("button.danger").trigger("click");
    await flushPromises();

    // Assert
    expect(wrapper.find(".error").text()).toBe("errore di scrittura");
    expect(wrapper.findAll("li")).toHaveLength(1);
  });

  it("clears a previous error when another entry is selected", async () => {
    // Arrange
    listEntries.mockResolvedValue([entry("1", "GitHub")]);
    saveEntry.mockRejectedValue("errore");
    const wrapper = await mountApp();
    await wrapper.find("li").trigger("click");
    await wrapper.find("form").trigger("submit");
    await flushPromises();

    // Act
    await wrapper.find("li").trigger("click");

    // Assert
    expect(wrapper.find(".error").exists()).toBe(false);
  });

  describe("vault lock", () => {
    const fillPasswords = async (wrapper, values) => {
      const inputs = wrapper.findAll("input[type=password]");
      for (const [i, v] of values.entries()) await inputs[i].setValue(v);
    };

    it("does not load entries while the vault is locked", async () => {
      // Arrange
      vaultStatus.mockResolvedValue("locked");

      // Act
      const wrapper = await mountApp();

      // Assert
      expect(listEntries).not.toHaveBeenCalled();
      expect(wrapper.text()).toContain("Vault locked");
    });

    it("shows the setup screen on first launch", async () => {
      // Arrange
      vaultStatus.mockResolvedValue("no_vault");

      // Act
      const wrapper = await mountApp();

      // Assert
      expect(wrapper.text()).toContain("Create the master password");
    });

    it("switches the interface to Italian from the unlock screen", async () => {
      // Arrange
      vaultStatus.mockResolvedValue("locked");
      const wrapper = await mountApp();

      // Act
      await wrapper.find("select").setValue("it");

      // Assert
      expect(wrapper.text()).toContain("Vault bloccato");
      expect(wrapper.find("button[type=submit]").text()).toBe("Sblocca");
    });

    it("shows a backend error code translated in the current language", async () => {
      // Arrange
      vaultStatus.mockResolvedValue("locked");
      unlockVault.mockRejectedValue("bad_password");
      const wrapper = await mountApp();
      await wrapper.find("select").setValue("it");

      // Act
      await wrapper.find("input").setValue("master-password-1");
      await wrapper.find("form").trigger("submit");
      await flushPromises();

      // Assert
      expect(wrapper.find(".error").text()).toBe("Password errata o vault danneggiato");
    });

    it("shows the migration screen for a plaintext vault", async () => {
      // Arrange
      vaultStatus.mockResolvedValue("legacy_plaintext");

      // Act
      const wrapper = await mountApp();

      // Assert
      expect(wrapper.text()).toContain("Protect the vault");
    });

    it("unlocks with the typed password, then loads entries", async () => {
      // Arrange
      vaultStatus.mockResolvedValue("locked");
      listEntries.mockResolvedValue([entry("1", "GitHub")]);
      const wrapper = await mountApp();
      await fillPasswords(wrapper, ["master-password-1"]);

      // Act
      await wrapper.find("form").trigger("submit");
      await flushPromises();

      // Assert
      expect(unlockVault).toHaveBeenCalledWith("master-password-1");
      expect(wrapper.findAll("li")).toHaveLength(1);
    });

    it("shows the error and stays locked when the password is wrong", async () => {
      // Arrange
      vaultStatus.mockResolvedValue("locked");
      unlockVault.mockRejectedValue("bad_password");
      const wrapper = await mountApp();
      await fillPasswords(wrapper, ["sbagliata"]);

      // Act
      await wrapper.find("form").trigger("submit");
      await flushPromises();

      // Assert
      expect(wrapper.find(".error").text()).toBe("Wrong password or damaged vault");
      expect(listEntries).not.toHaveBeenCalled();
    });

    it("creates the vault on first launch and then loads entries", async () => {
      // Arrange
      vaultStatus.mockResolvedValue("no_vault");
      const wrapper = await mountApp();
      await fillPasswords(wrapper, ["master-password-1", "master-password-1"]);

      // Act
      await wrapper.find("form").trigger("submit");
      await flushPromises();

      // Assert
      expect(setupVault).toHaveBeenCalledWith("master-password-1");
      expect(listEntries).toHaveBeenCalled();
    });

    it("lock button clears entries and returns to the unlock screen", async () => {
      // Arrange
      listEntries.mockResolvedValue([entry("1", "GitHub")]);
      const wrapper = await mountApp();
      await wrapper.find("li").trigger("click");

      // Act
      const lockButton = wrapper.findAll("button").find((b) => b.text() === "Lock");
      await lockButton.trigger("click");
      await flushPromises();

      // Assert
      expect(lockVault).toHaveBeenCalled();
      expect(wrapper.text()).toContain("Vault locked");
      expect(wrapper.find("li").exists()).toBe(false);
      expect(wrapper.text()).not.toContain("GitHub");
    });

    it("stays unlocked and shows the error when locking fails", async () => {
      // Arrange
      lockVault.mockRejectedValue("errore lock");
      listEntries.mockResolvedValue([entry("1", "GitHub")]);
      const wrapper = await mountApp();

      // Act
      const lockButton = wrapper.findAll("button").find((b) => b.text() === "Lock");
      await lockButton.trigger("click");
      await flushPromises();

      // Assert
      expect(wrapper.find(".error").text()).toBe("errore lock");
      expect(wrapper.findAll("li")).toHaveLength(1);
    });

    it("changes the master password from the dialog and closes it", async () => {
      // Arrange
      const wrapper = await mountApp();
      const open = wrapper.findAll("button").find((b) => b.text() === "Change password");
      await open.trigger("click");
      const dialog = wrapper.find("[role=dialog]");
      const inputs = dialog.findAll("input");
      await inputs[0].setValue("master-password-1");
      await inputs[1].setValue("nuova-password-12");
      await inputs[2].setValue("nuova-password-12");

      // Act
      await dialog.trigger("submit");
      await flushPromises();

      // Assert
      expect(changePassword).toHaveBeenCalledWith("master-password-1", "nuova-password-12");
      expect(wrapper.find("[role=dialog]").exists()).toBe(false);
    });

    it("keeps the dialog open and shows the backend error on failure", async () => {
      // Arrange
      changePassword.mockRejectedValue("wrong_current_password");
      const wrapper = await mountApp();
      const open = wrapper.findAll("button").find((b) => b.text() === "Change password");
      await open.trigger("click");
      const dialog = wrapper.find("[role=dialog]");
      const inputs = dialog.findAll("input");
      await inputs[0].setValue("sbagliata");
      await inputs[1].setValue("nuova-password-12");
      await inputs[2].setValue("nuova-password-12");

      // Act
      await dialog.trigger("submit");
      await flushPromises();

      // Assert
      expect(wrapper.find("[role=dialog] .error").text()).toBe("The current password is not correct");
    });

    it("shows the unlock screen when the status check fails", async () => {
      // Arrange
      vaultStatus.mockRejectedValue("errore di lettura");

      // Act
      const wrapper = await mountApp();

      // Assert
      expect(wrapper.find(".error").text()).toBe("errore di lettura");
    });
  });
});
