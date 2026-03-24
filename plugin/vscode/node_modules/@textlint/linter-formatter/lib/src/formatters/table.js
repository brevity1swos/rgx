/**
 * @fileoverview "table reporter.
 * @author Gajus Kuizinas <gajus@gajus.com>
 * @copyright 2016 Gajus Kuizinas <gajus@gajus.com>. All rights reserved.
 */
"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
//------------------------------------------------------------------------------
// Requirements
//------------------------------------------------------------------------------
const chalk_1 = __importDefault(require("chalk"));
const table_1 = require("table");
// @ts-expect-error no types
const pluralize_1 = __importDefault(require("pluralize"));
const strip_ansi_1 = __importDefault(require("strip-ansi"));
//------------------------------------------------------------------------------
// Helpers
//------------------------------------------------------------------------------
/**
 * Draws text table.
 * @param {Array<Object>} messages Error messages relating to a specific file.
 * @returns {string} A text table.
 */
function drawTable(messages) {
    const rows = [];
    if (messages.length === 0) {
        return "";
    }
    rows.push([
        chalk_1.default.bold("Line"),
        chalk_1.default.bold("Column"),
        chalk_1.default.bold("Type"),
        chalk_1.default.bold("Message"),
        chalk_1.default.bold("Rule ID")
    ]);
    messages.forEach(function (message) {
        let messageType;
        const fatal = message.fatal;
        if (fatal || message.severity === 2) {
            messageType = chalk_1.default.red("error");
        }
        else if (message.severity === 1) {
            messageType = chalk_1.default.yellow("warning");
        }
        else if (message.severity === 3) {
            messageType = chalk_1.default.green("info");
        }
        else {
            messageType = chalk_1.default.yellow("warning");
        }
        rows.push([message.line || 0, message.column || 0, messageType, message.message, message.ruleId || ""]);
    });
    const output = (0, table_1.table)(rows, {
        columns: {
            0: {
                width: 8,
                wrapWord: true
            },
            1: {
                width: 8,
                wrapWord: true
            },
            2: {
                width: 8,
                wrapWord: true
            },
            3: {
                paddingRight: 5,
                width: 50,
                wrapWord: true
            },
            4: {
                width: 20,
                wrapWord: true
            }
        },
        drawHorizontalLine(index) {
            return index === 1;
        }
    });
    return output;
}
/**
 * Draws a report (multiple tables).
 * @param {Array} results Report results for every file.
 * @returns {string} A column of text tables.
 */
function drawReport(results) {
    let files;
    files = results.map(function (result) {
        if (!result.messages.length) {
            return "";
        }
        return `\n${result.filePath}\n\n${drawTable(result.messages)}`;
    });
    files = files.filter(function (content) {
        return content.trim();
    });
    return files.join("");
}
//------------------------------------------------------------------------------
// Public Interface
//------------------------------------------------------------------------------
function formatter(report, options) {
    // default: true
    const useColor = options.color !== undefined ? options.color : true;
    let output = "";
    let errorCount = 0;
    let warningCount = 0;
    let infoCount = 0;
    report.forEach(function (fileReport) {
        fileReport.messages.forEach(function (message) {
            const fatal = message.fatal;
            if (fatal || message.severity === 2) {
                errorCount++;
            }
            else if (message.severity === 1) {
                warningCount++;
            }
            else if (message.severity === 3) {
                infoCount++;
            }
        });
    });
    if (errorCount || warningCount || infoCount) {
        output = drawReport(report);
    }
    const tableData = [];
    if (errorCount > 0) {
        tableData.push([chalk_1.default.red((0, pluralize_1.default)("Error", errorCount, true))]);
    }
    if (warningCount > 0) {
        tableData.push([chalk_1.default.yellow((0, pluralize_1.default)("Warning", warningCount, true))]);
    }
    if (infoCount > 0) {
        tableData.push([chalk_1.default.green((0, pluralize_1.default)("Info", infoCount, true))]);
    }
    if (tableData.length > 0) {
        output += `\n${(0, table_1.table)(tableData, {
            columns: {
                0: {
                    width: 110,
                    wrapWord: true
                }
            },
            drawHorizontalLine() {
                return true;
            }
        })}`;
    }
    if (!useColor) {
        return (0, strip_ansi_1.default)(output);
    }
    return output;
}
exports.default = formatter;
//# sourceMappingURL=table.js.map