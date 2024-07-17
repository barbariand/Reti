import { describe, expect, test } from "vitest";
import { render, screen } from "@testing-library/svelte";
import KaTeX from "./KaTeX.svelte";

describe("KaTeX", () => {
    test("Render text", () => {
        render(KaTeX, { latex: "abc" });
        expect(screen.getByText(/abc/)).toBeInTheDocument();
    });

    test("KaTeX has aria-label", () => {
        render(KaTeX, { latex: "\\frac{2\\pi}{3}" });
        expect(screen.getByLabelText("\\frac{2\\pi}{3}")).toBeInTheDocument();
    });
});
