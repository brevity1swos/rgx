"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.activate = activate;
exports.deactivate = deactivate;
const vscode = require("vscode");
function getConfig() {
    const config = vscode.workspace.getConfiguration("rgx");
    return {
        path: config.get("path", "rgx"),
        defaultEngine: config.get("defaultEngine", ""),
    };
}
function getSelectedText() {
    const editor = vscode.window.activeTextEditor;
    if (!editor)
        return "";
    const selection = editor.selection;
    return editor.document.getText(selection);
}
function shellEscape(arg) {
    if (process.platform === "win32") {
        return `'${arg.replace(/'/g, "''")}'`;
    }
    return `'${arg.replace(/'/g, "'\\''")}'`;
}
function buildCommand(args) {
    const { path, defaultEngine } = getConfig();
    const parts = [path];
    if (defaultEngine) {
        parts.push("--engine", defaultEngine);
    }
    parts.push(...args);
    return parts.join(" ");
}
function runInTerminal(cmd) {
    const terminal = vscode.window.createTerminal({ name: "rgx" });
    terminal.sendText(cmd);
    terminal.show();
}
function activate(context) {
    context.subscriptions.push(vscode.commands.registerCommand("rgx.open", () => {
        runInTerminal(buildCommand([]));
    }), vscode.commands.registerCommand("rgx.openWithSelection", () => {
        const text = getSelectedText();
        const args = text ? ["--text", shellEscape(text)] : [];
        runInTerminal(buildCommand(args));
    }), vscode.commands.registerCommand("rgx.openWithPattern", () => {
        const text = getSelectedText();
        const args = text ? [shellEscape(text)] : [];
        runInTerminal(buildCommand(args));
    }));
}
function deactivate() { }
//# sourceMappingURL=extension.js.map