// LICENSE : MIT
// Original code is https://github.com/azer/prettify-error
// Author : azer
"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.prettyError = prettyError;
const chalk_1 = __importDefault(require("chalk"));
// @ts-expect-error no types
const format_text_1 = __importDefault(require("@azu/format-text"));
// @ts-expect-error no types
const style_format_1 = __importDefault(require("@azu/style-format"));
const strip_ansi_1 = __importDefault(require("strip-ansi"));
// @ts-expect-error no types
const pluralize_1 = __importDefault(require("pluralize"));
// width is 2
const string_width_1 = __importDefault(require("string-width"));
// color set
const node_fs_1 = require("node:fs");
const summaryColor = "yellow";
const greenColor = "green";
const template = (0, style_format_1.default)("{grey}{ruleId}: {red}{title}{reset}\n" +
    "{grey}{filename}{reset}\n" +
    "    {red}{paddingForLineNo}  {v}{reset}\n" +
    "    {grey}{previousLineNo}. {previousLine}{reset}\n" +
    "    {reset}{failingLineNo}. {failingLine}{reset}\n" +
    "    {grey}{nextLineNo}. {nextLine}{reset}\n" +
    "    {red}{paddingForLineNo}  {^}{reset}\n" +
    "");
/**
 *
 * @param {string} code
 * @param {TextLintMessage} message
 * @returns {*}
 */
function failingCode(code, message) {
    const result = [];
    const lines = code.split("\n");
    let i = message.line - 3;
    while (++i < message.line + 1) {
        if (i + 1 !== message.line) {
            result.push({
                line: message.line - (message.line - i - 1),
                code: lines[i]
            });
            continue;
        }
        result.push({
            line: message.line,
            col: message.column,
            code: lines[i],
            failed: true
        });
    }
    return result;
}
function showColumn(codes, ch) {
    let result = "";
    const codeObject = codes[1];
    const sliced = codeObject.code.slice(0, codeObject.col);
    const width = (0, string_width_1.default)(sliced);
    if (width <= 0) {
        return "";
    }
    let i = width - 1;
    while (i--) {
        result += " ";
    }
    return result + ch;
}
/**
 *
 * @param {string} code
 * @param {string} filePath
 * @param {TextLintMessage} message
 * @returns {*}
 */
function prettyError(code, filePath, message) {
    if (!code) {
        return;
    }
    const parsed = failingCode(code, message);
    const previousLineNo = String(parsed[0].line);
    const failingLineNo = String(parsed[1].line);
    const nextLineNo = String(parsed[2].line);
    const linumlen = Math.max(previousLineNo.length, failingLineNo.length, nextLineNo.length);
    return (0, format_text_1.default)(template, {
        ruleId: message.ruleId,
        title: message.message,
        filename: `${filePath}:${message.line}:${message.column}`,
        previousLine: parsed[0].code ? parsed[0].code : "",
        previousLineNo: previousLineNo.padStart(linumlen),
        previousColNo: parsed[0].col,
        failingLine: parsed[1].code,
        failingLineNo: failingLineNo.padStart(linumlen),
        failingColNo: parsed[1].col,
        nextLine: parsed[2].code ? parsed[2].code : "",
        nextLineNo: nextLineNo.padStart(linumlen),
        nextColNo: parsed[2].col,
        paddingForLineNo: "".padStart(linumlen),
        "^": showColumn(parsed, "^"),
        v: showColumn(parsed, "v")
    });
}
/**
 *
 * @param {TextLintResult[]} results
 * @param {TextLintFormatterOption} options
 * @returns {string}
 */
function formatter(results, options) {
    // default: true
    const useColor = options.color !== undefined ? options.color : true;
    let output = "";
    let total = 0;
    let errors = 0;
    let warnings = 0;
    let infos = 0;
    let totalFixable = 0;
    results.forEach(function (result) {
        const code = (0, node_fs_1.readFileSync)(result.filePath, "utf-8");
        const messages = result.messages;
        if (messages.length === 0) {
            return;
        }
        total += messages.length;
        messages.forEach(function (message) {
            // fixable
            const fixableIcon = message.fix ? chalk_1.default[greenColor].bold("\u2713 ") : "";
            if (message.fix) {
                totalFixable++;
            }
            const fatal = message.fatal;
            if (fatal || message.severity === 2) {
                errors++;
            }
            else if (message.severity === 1) {
                warnings++;
            }
            else if (message.severity === 3) {
                infos++;
            }
            else {
                warnings++;
            }
            const r = fixableIcon + prettyError(code, result.filePath, message);
            if (r) {
                output += `${r}\n`;
            }
        });
    });
    if (total > 0) {
        const problemParts = [];
        if (errors > 0) {
            problemParts.push(`${errors} ${(0, pluralize_1.default)("error", errors)}`);
        }
        if (warnings > 0) {
            problemParts.push(`${warnings} ${(0, pluralize_1.default)("warning", warnings)}`);
        }
        if (infos > 0) {
            problemParts.push(`${infos} ${(0, pluralize_1.default)("info", infos)}`);
        }
        output += chalk_1.default[summaryColor].bold(["\u2716 ", total, (0, pluralize_1.default)(" problem", total), " (", problemParts.join(", "), ")\n"].join(""));
    }
    if (totalFixable > 0) {
        output += chalk_1.default[greenColor].bold(`✓ ${totalFixable} fixable ${(0, pluralize_1.default)("problem", totalFixable)}.\n`);
        output += `Try to run: $ ${chalk_1.default.underline("textlint --fix [file]")}\n`;
    }
    if (!useColor) {
        return (0, strip_ansi_1.default)(output);
    }
    return output;
}
exports.default = formatter;
//# sourceMappingURL=pretty-error.js.map