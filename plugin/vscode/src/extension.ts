import * as vscode from "vscode";

function getConfig() {
  const config = vscode.workspace.getConfiguration("rgx");
  return {
    path: config.get<string>("path", "rgx"),
    defaultEngine: config.get<string>("defaultEngine", ""),
  };
}

function getSelectedText(): string {
  const editor = vscode.window.activeTextEditor;
  if (!editor) return "";
  const selection = editor.selection;
  return editor.document.getText(selection);
}

function shellEscape(arg: string): string {
  if (process.platform === "win32") {
    return `'${arg.replace(/'/g, "''")}'`;
  }
  return `'${arg.replace(/'/g, "'\\''")}'`;
}

function buildCommand(args: string[]): string {
  const { path, defaultEngine } = getConfig();
  const parts = [path];
  if (defaultEngine) {
    parts.push("--engine", defaultEngine);
  }
  parts.push(...args);
  return parts.join(" ");
}

function runInTerminal(cmd: string) {
  const terminal = vscode.window.createTerminal({ name: "rgx" });
  terminal.sendText(cmd);
  terminal.show();
}

export function activate(context: vscode.ExtensionContext) {
  context.subscriptions.push(
    vscode.commands.registerCommand("rgx.open", () => {
      runInTerminal(buildCommand([]));
    }),
    vscode.commands.registerCommand("rgx.openWithSelection", () => {
      const text = getSelectedText();
      const args = text ? ["--text", shellEscape(text)] : [];
      runInTerminal(buildCommand(args));
    }),
    vscode.commands.registerCommand("rgx.openWithPattern", () => {
      const text = getSelectedText();
      const args = text ? [shellEscape(text)] : [];
      runInTerminal(buildCommand(args));
    })
  );
}

export function deactivate() {}
