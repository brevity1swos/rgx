/**
 * @fileoverview jUnit Reporter
 * @author Jamund Ferguson
 */
"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const lodash_1 = __importDefault(require("lodash"));
//------------------------------------------------------------------------------
// Helper Functions
//------------------------------------------------------------------------------
function getMessageType(message) {
    if (message.fatal || message.severity === 2) {
        return "Error";
    }
    else if (message.severity === 1) {
        return "Warning";
    }
    else if (message.severity === 3) {
        return "Info";
    }
    else {
        return "Warning";
    }
}
//------------------------------------------------------------------------------
// Public Interface
//------------------------------------------------------------------------------
function formatter(results) {
    let output = "";
    output += '<?xml version="1.0" encoding="utf-8"?>\n';
    output += "<testsuites>\n";
    results.forEach(function (result) {
        const messages = result.messages;
        if (messages.length) {
            output += `<testsuite package="org.eslint" time="0" tests="${messages.length}" errors="${messages.length}" name="${result.filePath}">\n`;
        }
        messages.forEach(function (message) {
            const fatal = message.fatal;
            const type = fatal ? "error" : "failure";
            output += `<testcase time="0" name="org.eslint.${message.ruleId || "unknown"}">`;
            output += `<${type} message="${lodash_1.default.escape(message.message || "")}">`;
            output += "<![CDATA[";
            output += `line ${message.line || 0}, col `;
            output += `${message.column || 0}, ${getMessageType(message)}`;
            output += ` - ${lodash_1.default.escape(message.message || "")}`;
            output += message.ruleId ? ` (${message.ruleId})` : "";
            output += "]]>";
            output += `</${type}>`;
            output += "</testcase>\n";
        });
        if (messages.length) {
            output += "</testsuite>\n";
        }
    });
    output += "</testsuites>\n";
    return output;
}
exports.default = formatter;
//# sourceMappingURL=junit.js.map