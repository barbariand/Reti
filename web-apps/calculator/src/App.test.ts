import { describe, expect, test } from "vitest";
import { render, screen } from "@testing-library/svelte";
import App from "./App.svelte";

describe("App", () => {
    test("Has (1) row", () => {
        render(App);
        expect(screen.getByText(/\(1\)/)).toBeInTheDocument();
    });
});
