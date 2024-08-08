<script lang="ts">
    import KaTeX from "./KaTeX.svelte";

    import {
        RetiJS,
        type AstError,
        type RetiJsError,
        type RetiJsErrorEnum,
        type RetiJsEvaluation,
    } from "reti-js";

    type RetiJsResult = {
        Evaluation?: RetiJsEvaluation;
        Error?: RetiJsError;
    };
    export let reti: RetiJS;
    export let rowNumber: number;
    export let on_first_input: () => void = () => {};
    let latex = "";
    $: result = parse(latex);

    function parse(latex: string): RetiJsResult | null {
        if (latex == "") {
            return null;
        }
        try {
            let res: RetiJsEvaluation = reti.parse(latex);
            console.log("res:", res, "typeof:", typeof res);
            return { Evaluation: res };
        } catch (e: unknown) {
            if (isRetiJsError(e)) {
                return { Error: e };
            } else {
                console.error(e);
                return {
                    Error: {
                        display: "Evaluation probably paniced",
                        error: "Unknown",
                    },
                };
            }
        }
    }
    function isRetiJsError(error: unknown): error is RetiJsError {
        return (
            typeof error === "object" &&
            error !== null &&
            "display" in error &&
            typeof error.display === "string" &&
            "error" in error &&
            isRetiJsErrorEnum(error.error)
        );
    }
    function isRetiJsErrorEnum(error: unknown): error is RetiJsErrorEnum {
        return (
            (typeof error === "object" &&
                error !== null &&
                (("EvalError" in error && isEvalError(error.EvalError)) ||
                    ("AstError" in error && isAstError(error.AstError)))) ||
            error == "Unknown"
        );
    }

    function isEvalError(error: unknown): error is EvalError {
        if (typeof error === "string") {
            return ["ExpectedScalar", "NotDefined", "DivideByZero"].includes(
                error,
            );
        }

        if (typeof error === "object" && error !== null) {
            return (
                "IncompatibleTypes" in error ||
                "IncompatibleMatrixSizes" in error ||
                "AmbiguousMulType" in error ||
                "ArgumentLengthMismatch" in error ||
                "DeriveError" in error
            );
        }

        return false;
    }

    function isAstError(error: unknown): error is AstError {
        if (typeof error === "object" && error !== null) {
            return "Join" in error || "Panic" in error || "ParseError" in error;
        }

        return false;
    }
    let is_first = true;
    function handleInput() {
        if (is_first) {
            on_first_input();
            is_first = false;
        }
    }
</script>

<div class="calculator-row">
    <div class="number">({rowNumber})</div>
    <div class="calculator-row-main">
        <div class="input-container">
            <textarea
                class="input"
                bind:value={latex}
                on:input={handleInput}
                rows="1"
            />
        </div>
        <div class="math-container">
            <div class="math-input">
                <KaTeX display {latex} />
            </div>
            <div class="math-output">
                {#if result?.Evaluation}
                    <p>
                        {#if result.Evaluation.tag === "AddedFunction"}
                            Added Function:
                        {:else if result.Evaluation.tag === "AddedVariable"}
                            Added Variable:
                        {/if}
                    </p>
                    <KaTeX display latex={result.Evaluation.latex} />
                {:else if result?.Error}
                    <span class="error">{result.Error.display}</span>
                {/if}
            </div>
        </div>
    </div>
</div>

<style>
    .calculator-row {
        display: flex;
        gap: 10px;
    }
    .calculator-row-main {
        width: 100%;
        display: flex;
        flex-direction: column;
        gap: 10px;
    }
    .input-container {
        display: flex;
        justify-content: stretch;
    }
    .input {
        padding: 8px;
        border-radius: 5px;
        border: 1px solid #7dedc4;
        font-family: monospace;
        line-height: 15px;
        font-size: 14px;
        flex-grow: 1;
        resize: vertical;
    }
    .math-container {
        display: flex;
        flex-wrap: wrap;
        justify-content: space-between;
        align-items: center;
    }
    .math-output {
        justify-self: end, flex-end;
        display: flex;
        justify-content: center;
    }

    .error {
        color: red;
    }
</style>
