import { describe, expect, test } from "vitest";
import { render, screen } from "@testing-library/svelte";
import App from "./App.svelte";

describe("App", () => {
    test("has powered by text", () => {
        render(App);
        expect(screen.getByText(/powered by Vite/)).toBeInTheDocument();
    });

    test("no Reti branding", () => {
        render(App);
        expect(screen.queryByText(/reti/i)).not.toBeInTheDocument();
    });
});
