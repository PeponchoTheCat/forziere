import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { mount } from "@vue/test-utils";

vi.mock("../api", () => ({ generatePassword: vi.fn() }));

import { generatePassword } from "../api";
import EntryForm from "./EntryForm.vue";

const existing = {
  id: "id-1",
  service: "GitHub",
  url: "https://github.com",
  username: "mario",
  password: "pw",
  notes: "nota",
  tags: ["dev", "lavoro"],
  created_at: "2026-10-23T00:00:00Z",
  updated_at: "2026-10-23T00:00:00Z",
};

// ordine nel DOM: servizio, url, utente, password, [generatore], tag, note
const field = (wrapper, label) =>
  wrapper.findAll("label").find((l) => l.text().startsWith(label)).find("input, textarea");
const passwordInput = (wrapper) => field(wrapper, "Password");
const button = (wrapper, text) => wrapper.findAll("button").find((b) => b.text() === text);

describe("EntryForm", () => {
  let writeText;

  beforeEach(() => {
    writeText = vi.fn().mockResolvedValue(undefined);
    Object.defineProperty(navigator, "clipboard", { value: { writeText }, configurable: true });
    generatePassword.mockReset();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("shows an empty form titled 'Nuova voce' when entry is null", () => {
    // Arrange & Act
    const wrapper = mount(EntryForm, { props: { entry: null } });

    // Assert
    expect(wrapper.find("h2").text()).toBe("Nuova voce");
    expect(field(wrapper, "Servizio").element.value).toBe("");
    expect(wrapper.text()).not.toContain("Elimina");
  });

  it("prefills fields and joins tags when editing an entry", () => {
    // Arrange & Act
    const wrapper = mount(EntryForm, { props: { entry: existing } });

    // Assert
    expect(wrapper.find("h2").text()).toBe("Modifica voce");
    expect(field(wrapper, "Servizio").element.value).toBe("GitHub");
    expect(field(wrapper, "URL").element.value).toBe("https://github.com");
    expect(field(wrapper, "Utente").element.value).toBe("mario");
    expect(passwordInput(wrapper).element.value).toBe("pw");
    expect(field(wrapper, "Tag").element.value).toBe("dev, lavoro");
    expect(field(wrapper, "Note").element.value).toBe("nota");
  });

  it("reloads the form when the entry prop changes", async () => {
    // Arrange
    const wrapper = mount(EntryForm, { props: { entry: existing } });

    // Act
    await wrapper.setProps({ entry: null });

    // Assert
    expect(field(wrapper, "Servizio").element.value).toBe("");
  });

  it("hides the password by default and toggles visibility", async () => {
    // Arrange
    const wrapper = mount(EntryForm, { props: { entry: existing } });
    const initialType = passwordInput(wrapper).attributes("type");

    // Act
    await button(wrapper, "Mostra").trigger("click");

    // Assert
    expect(initialType).toBe("password");
    expect(passwordInput(wrapper).attributes("type")).toBe("text");
    expect(button(wrapper, "Nascondi")).toBeDefined();
  });

  it("resets password visibility when switching entry", async () => {
    // Arrange
    const wrapper = mount(EntryForm, { props: { entry: existing } });
    await button(wrapper, "Mostra").trigger("click");

    // Act
    await wrapper.setProps({ entry: { ...existing, id: "id-2" } });

    // Assert
    expect(passwordInput(wrapper).attributes("type")).toBe("password");
  });

  it("copies the password to the clipboard and shows feedback that then fades", async () => {
    // Arrange
    vi.useFakeTimers();
    const wrapper = mount(EntryForm, { props: { entry: existing } });

    // Act
    await button(wrapper, "Copia").trigger("click");
    await vi.advanceTimersByTimeAsync(0);
    const during = wrapper.text();
    await vi.advanceTimersByTimeAsync(1600);

    // Assert
    expect(writeText).toHaveBeenCalledWith("pw");
    expect(during).toContain("Copiata!");
    expect(wrapper.text()).not.toContain("Copiata!");
  });

  it("does not show copy feedback when the clipboard write fails", async () => {
    // Arrange
    writeText.mockRejectedValue(new Error("denied"));
    const wrapper = mount(EntryForm, { props: { entry: existing } });

    // Act
    await button(wrapper, "Copia").trigger("click");
    await new Promise((r) => setTimeout(r, 0));

    // Assert
    expect(wrapper.text()).not.toContain("Copiata!");
  });

  it("disables the copy button when the password is empty", () => {
    // Arrange & Act
    const wrapper = mount(EntryForm, { props: { entry: null } });

    // Assert
    expect(button(wrapper, "Copia").attributes("disabled")).toBeDefined();
  });

  it("fills the password and reveals it when the generator emits use", async () => {
    // Arrange
    const wrapper = mount(EntryForm, { props: { entry: null } });

    // Act
    wrapper.findComponent({ name: "PasswordGenerator" }).vm.$emit("use", "Gen#123");
    await wrapper.vm.$nextTick();

    // Assert
    expect(passwordInput(wrapper).element.value).toBe("Gen#123");
    expect(passwordInput(wrapper).attributes("type")).toBe("text");
  });

  it("emits save with parsed tags and null id for a new entry", async () => {
    // Arrange
    const wrapper = mount(EntryForm, { props: { entry: null } });
    await field(wrapper, "Servizio").setValue("GitLab");
    await field(wrapper, "Utente").setValue("luigi");
    await passwordInput(wrapper).setValue("p4ss");
    await field(wrapper, "Tag").setValue(" a , ,b ");

    // Act
    await wrapper.find("form").trigger("submit");

    // Assert
    expect(wrapper.emitted("save")).toEqual([
      [{ id: null, service: "GitLab", url: "", username: "luigi", password: "p4ss", notes: "", tags: ["a", "b"] }],
    ]);
  });

  it("emits save keeping the id when editing, without timestamps", async () => {
    // Arrange
    const wrapper = mount(EntryForm, { props: { entry: existing } });

    // Act
    await wrapper.find("form").trigger("submit");

    // Assert
    const [payload] = wrapper.emitted("save")[0];
    expect(payload.id).toBe("id-1");
    expect(payload.tags).toEqual(["dev", "lavoro"]);
    expect(payload).not.toHaveProperty("created_at");
    expect(payload).not.toHaveProperty("updated_at");
  });

  it("emits delete with the entry id", async () => {
    // Arrange
    const wrapper = mount(EntryForm, { props: { entry: existing } });

    // Act
    await button(wrapper, "Elimina").trigger("click");

    // Assert
    expect(wrapper.emitted("delete")).toEqual([["id-1"]]);
  });

  it("renders the error prop when provided", () => {
    // Arrange & Act
    const wrapper = mount(EntryForm, { props: { entry: null, error: "Servizio e utente sono obbligatori" } });

    // Assert
    expect(wrapper.find(".error").text()).toBe("Servizio e utente sono obbligatori");
  });

  it("does not render an error element when there is no error", () => {
    // Arrange & Act
    const wrapper = mount(EntryForm, { props: { entry: null } });

    // Assert
    expect(wrapper.find(".error").exists()).toBe(false);
  });

  it("marks service and username as required", () => {
    // Arrange & Act
    const wrapper = mount(EntryForm, { props: { entry: null } });

    // Assert
    expect(field(wrapper, "Servizio").attributes("required")).toBeDefined();
    expect(field(wrapper, "Utente").attributes("required")).toBeDefined();
  });
});
