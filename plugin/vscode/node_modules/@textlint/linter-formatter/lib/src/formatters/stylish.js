/**
 * @fileoverview Stylish reporter
 * @author Sindre Sorhus
 */
"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const chalk_1 = __importDefault(require("chalk"));
// @ts-expect-error no types
const text_table_1 = __importDefault(require("text-table"));
const string_width_1 = __importDefault(require("string-width"));
const strip_ansi_1 = __importDefault(require("strip-ansi"));
//------------------------------------------------------------------------------
// Helpers
//------------------------------------------------------------------------------
/**
 * Given a word and a count, append an s if count is not one.
 * @param {string} word A word in its singular form.
 * @param {int} count A number controlling whether word should be pluralized.
 * @returns {string} The original word with an s on the end if count is not one.
 */
function pluralize(word, count) {
    return count === 1 ? word : `${word}s`;
}
//------------------------------------------------------------------------------
// Public Interface
//------------------------------------------------------------------------------
function formatter(results, options) {
    // default: true
    const useColor = options.color !== undefined ? options.color : true;
    let output = "\n";
    let total = 0;
    let totalFixable = 0;
    let errors = 0;
    let warnings = 0;
    let infos = 0;
    let summaryColor = "yellow";
    const greenColor = "green";
    results.forEach(function (result) {
        const messages = result.messages;
        if (messages.length === 0) {
            return;
        }
        total += messages.length;
        output += `${chalk_1.default.underline(result.filePath)}\n`;
        output += `${(0, text_table_1.default)(messages.map(function (message) {
            let messageType;
            // fixable
            const fixableIcon = message.fix ? chalk_1.default[greenColor].bold("\u2713 ") : "";
            if (message.fix) {
                totalFixable++;
            }
            const fatal = message.fatal;
            if (fatal || message.severity === 2) {
                messageType = fixableIcon + chalk_1.default.red("error");
                summaryColor = "red";
                errors++;
            }
            else if (message.severity === 1) {
                messageType = fixableIcon + chalk_1.default.yellow("warning");
                warnings++;
            }
            else if (message.severity === 3) {
                messageType = fixableIcon + chalk_1.default.green("info");
                infos++;
            }
            else {
                // fallback for other severity levels
                messageType = fixableIcon + chalk_1.default.yellow("warning");
                warnings++;
            }
            return [
                "",
                message.line || 0,
                message.column || 0,
                messageType,
                message.message.replace(/\.$/, ""),
                chalk_1.default.gray(message.ruleId || "")
            ];
        }), {
            align: ["", "r", "l"],
            stringLength(str) {
                const lines = (0, strip_ansi_1.default)(str).split("\n");
                return Math.max.apply(null, lines.map(function (line) {
                    return (0, string_width_1.default)(line);
                }));
            }
        })
            .split("\n")
            .map(function (el) {
            return el.replace(/(\d+)\s+(\d+)/, function (_, p1, p2) {
                return chalk_1.default.gray(`${p1}:${p2}`);
            });
        })
            .join("\n")}\n\n`;
    });
    if (total > 0) {
        const summaryParts = [
            `${errors} ${pluralize("error", errors)}`,
            `${warnings} ${pluralize("warning", warnings)}`,
            `${infos} ${pluralize("info", infos)}`
        ];
        output += chalk_1.default[summaryColor].bold(["\u2716 ", total, pluralize(" problem", total), " (", summaryParts.join(", "), ")\n"].join(""));
    }
    if (totalFixable > 0) {
        output += chalk_1.default[greenColor].bold(`✓ ${totalFixable} fixable ${pluralize("problem", totalFixable)}.\n`);
        output += `Try to run: $ ${chalk_1.default.underline("textlint --fix [file]")}\n`;
    }
    const finalOutput = total > 0 ? output : "";
    if (!useColor) {
        return (0, strip_ansi_1.default)(finalOutput);
    }
    return finalOutput;
}
exports.default = formatter;
//# sourceMappingURL=stylish.js.map