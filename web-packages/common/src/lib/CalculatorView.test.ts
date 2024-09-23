import { describe, expect, test } from "vitest";
import { render, screen } from "@testing-library/svelte";
import userEvent from "@testing-library/user-event";
import CalculatorView from "./CalculatorView.svelte";

describe("CalculatorView", () => {
    test("Rows (1) and (2) exist initially", () => {
        render(CalculatorView);
        expect(screen.getByText("(1)")).toBeInTheDocument();
        expect(screen.getByText("(2)")).toBeInTheDocument();
        expect(screen.queryByText("(3)")).not.toBeInTheDocument();
    });
    test("New row appears when pressing add row", async () => {
        const user = userEvent.setup();
        render(CalculatorView);

        const button = screen.getByText("Add row");
        await user.click(button);

        expect(screen.getByText("(3)")).toBeInTheDocument();
    });
    test("1+1=2", async () => {
        const user = userEvent.setup();
        render(CalculatorView);

        const inputs = screen.getAllByRole("textbox");
        expect(inputs.length).toBe(2);
        const input = inputs[0];
        expect(input).toBeInTheDocument();
        await user.type(input, "1+1");

        expect(screen.getByLabelText("=2")).toBeInTheDocument();
    });
});
