import { describe, expect, it } from "vitest";
import { mount } from "@vue/test-utils";
import EntryList from "./EntryList.vue";

const entry = (id, service, username = "user", tags = []) => ({ id, service, username, tags });

const entries = [
  entry("1", "Zeta", "alice", ["lavoro"]),
  entry("2", "alpha", "bob", ["personale"]),
  entry("3", "Mail", "carol", []),
];

const services = (wrapper) => wrapper.findAll("li strong").map((n) => n.text());

describe("EntryList", () => {
  it("renders entries sorted by service ignoring case", () => {
    // Arrange & Act
    const wrapper = mount(EntryList, { props: { entries } });

    // Assert
    expect(services(wrapper)).toEqual(["alpha", "Mail", "Zeta"]);
  });

  it("shows the empty message when there are no entries", () => {
    // Arrange & Act
    const wrapper = mount(EntryList, { props: { entries: [] } });

    // Assert
    expect(wrapper.find(".empty").exists()).toBe(true);
    expect(wrapper.findAll("li")).toHaveLength(0);
  });

  it("shows tags only for entries that have them", () => {
    // Arrange & Act
    const wrapper = mount(EntryList, { props: { entries } });

    // Assert
    expect(wrapper.findAll("li small")).toHaveLength(2);
  });

  it("filters by service name", async () => {
    // Arrange
    const wrapper = mount(EntryList, { props: { entries } });

    // Act
    await wrapper.find("input").setValue("ZET");

    // Assert
    expect(services(wrapper)).toEqual(["Zeta"]);
  });

  it("filters by username", async () => {
    // Arrange
    const wrapper = mount(EntryList, { props: { entries } });

    // Act
    await wrapper.find("input").setValue("bob");

    // Assert
    expect(services(wrapper)).toEqual(["alpha"]);
  });

  it("filters by tag", async () => {
    // Arrange
    const wrapper = mount(EntryList, { props: { entries } });

    // Act
    await wrapper.find("input").setValue("lavoro");

    // Assert
    expect(services(wrapper)).toEqual(["Zeta"]);
  });

  it("shows the empty message when the search has no match", async () => {
    // Arrange
    const wrapper = mount(EntryList, { props: { entries } });

    // Act
    await wrapper.find("input").setValue("nessuno");

    // Assert
    expect(wrapper.find(".empty").exists()).toBe(true);
  });

  it("ignores surrounding whitespace in the search query", async () => {
    // Arrange
    const wrapper = mount(EntryList, { props: { entries } });

    // Act
    await wrapper.find("input").setValue("  mail  ");

    // Assert
    expect(services(wrapper)).toEqual(["Mail"]);
  });

  it("marks the selected entry as active", () => {
    // Arrange & Act
    const wrapper = mount(EntryList, { props: { entries, selectedId: "3" } });

    // Assert
    const active = wrapper.findAll("li.active");
    expect(active).toHaveLength(1);
    expect(active[0].text()).toContain("Mail");
  });

  it("emits select with the entry id when an item is clicked", async () => {
    // Arrange
    const wrapper = mount(EntryList, { props: { entries } });

    // Act
    await wrapper.findAll("li")[0].trigger("click");

    // Assert
    expect(wrapper.emitted("select")).toEqual([["2"]]);
  });

  it("emits new when the new button is clicked", async () => {
    // Arrange
    const wrapper = mount(EntryList, { props: { entries } });

    // Act
    await wrapper.find("button").trigger("click");

    // Assert
    expect(wrapper.emitted("new")).toHaveLength(1);
  });

  it("does not mutate the entries prop when sorting", () => {
    // Arrange
    const original = [...entries];

    // Act
    mount(EntryList, { props: { entries } });

    // Assert
    expect(entries).toEqual(original);
  });
});
