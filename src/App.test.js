import { beforeEach, describe, expect, it, vi } from "vitest";
import { flushPromises, mount } from "@vue/test-utils";

vi.mock("./api", () => ({
  listEntries: vi.fn(),
  saveEntry: vi.fn(),
  deleteEntry: vi.fn(),
  generatePassword: vi.fn(),
}));

import { deleteEntry, listEntries, saveEntry } from "./api";
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
    expect(wrapper.find("h2").text()).toBe("Nuova voce");
  });

  it("opens the edit form when an entry is selected", async () => {
    // Arrange
    listEntries.mockResolvedValue([entry("1", "GitHub")]);
    const wrapper = await mountApp();

    // Act
    await wrapper.find("li").trigger("click");

    // Assert
    expect(wrapper.find("h2").text()).toBe("Modifica voce");
  });

  it("returns to an empty new-entry form when the new button is clicked", async () => {
    // Arrange
    listEntries.mockResolvedValue([entry("1", "GitHub")]);
    const wrapper = await mountApp();
    await wrapper.find("li").trigger("click");

    // Act
    await wrapper.find(".toolbar button").trigger("click");

    // Assert
    expect(wrapper.find("h2").text()).toBe("Nuova voce");
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
    expect(wrapper.find("h2").text()).toBe("Modifica voce");
  });

  it("shows the backend error when saving fails and keeps the list unchanged", async () => {
    // Arrange
    saveEntry.mockRejectedValue("Servizio e utente sono obbligatori");
    const wrapper = await mountApp();
    const inputs = wrapper.findAll("form input");
    await inputs[0].setValue("x");
    await inputs[2].setValue("y");

    // Act
    await wrapper.find("form").trigger("submit");
    await flushPromises();

    // Assert
    expect(wrapper.find(".error").text()).toBe("Servizio e utente sono obbligatori");
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
    expect(wrapper.find("h2").text()).toBe("Nuova voce");
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
});
