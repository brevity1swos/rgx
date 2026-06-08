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
    // VS Code's default integrated shell on Windows has been PowerShell
    // (pwsh / Windows PowerShell) since v1.65, where single-quote literals
    // with `''` doubling work correctly. But if the user has set their
    // default shell to cmd.exe, single quotes aren't special and the wrap
    // leaks the literal `'…'` as part of the argument (or worse, splits
    // the value on whitespace). Detect cmd.exe and switch to double-quote
    // wrapping with `""` doubling, which is cmd.exe's actual quoting rule.
    const shell = (vscode.env.shell || "").toLowerCase();
    if (shell.endsWith("cmd.exe") || shell.endsWith("\\cmd")) {
      return `"${arg.replace(/"/g, '""')}"`;
    }
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
    }),
    // Live-filter the current file's lines through `rgx filter`. Selection
    // (if any) becomes the pattern; the current file path becomes `--file`.
    // Unsaved/untitled buffers fall through to stdin so the user can pipe
    // their own input in the terminal.
    vscode.commands.registerCommand("rgx.filterFile", () => {
      const editor = vscode.window.activeTextEditor;
      const pattern = getSelectedText();
      const args: string[] = ["filter"];
      if (pattern) args.push(shellEscape(pattern));
      if (editor?.document.uri.scheme === "file") {
        args.push("--file", shellEscape(editor.document.uri.fsPath));
      }
      runInTerminal(buildCommand(args));
    })
  );
}

export function deactivate() {}
