/**
 * @fileoverview
 * Public function and interface should be included `Textlint` prefix or postfix.
 * It aim to avoid conflict naming on user land code.
 */
export type { TextlintSourceCode, TextlintSourceCodeArgs, TextlintSourceCodeLocation, TextlintSourceCodePosition, TextlintSourceCodeRange, } from "./Source/TextlintSourceCode.js";
export type { TextlintRulePaddingLocator } from "./Rule/TextlintRulePaddingLocator.js";
export type { TextlintRuleContextFixCommand } from "./Rule/TextlintRuleContextFixCommand.js";
export type { TextlintRuleContextFixCommandGenerator } from "./Rule/TextlintRuleContextFixCommandGenerator.js";
export type { TextlintRuleError, TextlintRuleErrorConstructor, TextlintRuleErrorDetails, TextlintRuleErrorPaddingLocation, TextlintRuleErrorPaddingLocationRange, TextlintRuleErrorPaddingLocationLoc, TextlintRuleReportedObject, } from "./Rule/TextlintRuleError.js";
export type { TextlintRuleSeverityLevel } from "./Rule/TextlintRuleSeverityLevel.js";
export type { TextlintRuleSeverityLevelKey } from "./Rule/TextlintRuleSeverityLevelKey.js";
export type { TextlintRuleSuggestion } from "./Rule/TextlintRuleSuggestion.js";
export type { TextlintRuleContext, TextlintRuleContextReportFunction, TextlintRuleContextReportFunctionArgs, } from "./Rule/TextlintRuleContext.js";
export type { TextlintRuleOptions } from "./Rule/TextlintRuleOptions.js";
export type { TextlintRuleReporter, TextlintFixableRuleModule, TextlintRuleModule, TextlintRuleReportHandler, } from "./Rule/TextlintRuleModule.js";
export type { TextlintFilterRuleContext, TextlintFilterRuleShouldIgnoreFunction, TextlintFilterRuleShouldIgnoreFunctionArgs, } from "./Rule/TextlintFilterRuleContext.js";
export type { TextlintFilterRuleModule, TextlintFilterRuleOptions, TextlintFilterRuleReporter, TextlintFilterRuleReportHandler, } from "./Rule/TextlintFilterRuleModule.js";
export type { TextlintPluginCreator, TextlintPluginOptions, TextlintPluginProcessor, TextlintPluginProcessorConstructor, TextlintPluginPreProcessResult, TextlintPluginPostProcessResult, } from "./Plugin/TextlintPluginModule.js";
export type { TextlintResult, TextlintFixResult, TextlintMessage, TextlintMessageFixCommand, TextlintMessageSuggestion, } from "./Message/TextlintResult.js";
//# sourceMappingURL=index.d.ts.map