import { describe, expect, test } from "vitest";
import { render, screen } from "@testing-library/svelte";
import App from "./App.svelte";

describe("App", () => {
    test("wip", () => {
        render(App);
        expect(screen.getByText(/work in progress/)).toBeInTheDocument();
    });
});
