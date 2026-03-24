import { TextlintRuleSeverityLevel } from "../Rule/TextlintRuleSeverityLevel.js";
export interface TextlintMessageFixCommand {
    text: string;
    range: readonly [startIndex: number, endIndex: number];
}
export interface TextlintMessageSuggestion {
    id: string;
    message: string;
    fix: TextlintMessageFixCommand;
}
export interface TextlintMessage {
    type: string;
    ruleId: string;
    message: string;
    data?: unknown;
    fix?: TextlintMessageFixCommand;
    /**
     * start line number where the issue is located.
     * start with 1
     * @deprecated use `loc`
     */
    line: number;
    /**
     * start column number where the issue is located.
     * start with 1
     * @deprecated use `loc`
     */
    column: number;
    /**
     * start index where the issue is located.
     * start with 0
     * @deprecated use `range`
     */
    index: number;
    /**
     * the range info where the issue is located.
     * range start with 0
     * Note: `range` represent same info to `loc`
     */
    range: readonly [startIndex: number, endIndex: number];
    /**
     * the location info where the issue is located.
     * line start with 1
     * column start with 1
     * Note: `loc` represent same info to `range`
     */
    loc: {
        start: {
            line: number;
            column: number;
        };
        end: {
            line: number;
            column: number;
        };
    };
    severity: TextlintRuleSeverityLevel;
    /**
     * List of suggestions to fix the issue.
     */
    suggestions?: TextlintMessageSuggestion[];
}
export interface TextlintResult {
    filePath: string;
    messages: TextlintMessage[];
}
export interface TextlintFixResult {
    filePath: string;
    output: string;
    messages: TextlintMessage[];
    applyingMessages: TextlintMessage[];
    remainingMessages: TextlintMessage[];
}
//# sourceMappingURL=TextlintResult.d.ts.map