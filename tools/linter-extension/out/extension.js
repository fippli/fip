"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.deactivate = exports.activate = void 0;
const child_process_1 = require("child_process");
const util_1 = require("util");
const vscode = require("vscode");
const execAsync = (0, util_1.promisify)(child_process_1.exec);
function activate(context) {
    const diagnosticCollection = vscode.languages.createDiagnosticCollection("fip");
    context.subscriptions.push(diagnosticCollection);
    const lintDocument = async (document) => {
        if (document.languageId !== "fip") {
            return;
        }
        const config = vscode.workspace.getConfiguration("fipLinter");
        if (!config.get("enable", true)) {
            diagnosticCollection.delete(document.uri);
            return;
        }
        const lintPath = config.get("path", "fip-lint");
        const filePath = document.uri.fsPath;
        console.log(`[FIP Linter] Linting ${filePath} with ${lintPath}`);
        try {
            const { stdout, stderr } = await execAsync(`${lintPath} "${filePath}"`, {
                timeout: 5000,
            });
            console.log(`[FIP Linter] stdout: ${stdout}`);
            console.log(`[FIP Linter] stderr: ${stderr}`);
            const diagnostics = [];
            const lines = (stdout + stderr).split("\n");
            for (const line of lines) {
                if (!line.trim()) {
                    continue;
                }
                // Parse format: file:line:column: severity: message
                const match = line.match(/^(.+?):(\d+):(\d+):\s+(error|warning|info):\s+(.+)$/);
                if (match) {
                    const [, , lineStr, colStr, severity, message] = match;
                    const lineNum = parseInt(lineStr, 10) - 1; // Convert to 0-based
                    const colNum = parseInt(colStr, 10) - 1;
                    const range = new vscode.Range(lineNum, colNum, lineNum, Math.max(colNum + 1, document.lineAt(lineNum).text.length));
                    let diagnosticSeverity;
                    switch (severity) {
                        case "error":
                            diagnosticSeverity = vscode.DiagnosticSeverity.Error;
                            break;
                        case "warning":
                            diagnosticSeverity = vscode.DiagnosticSeverity.Warning;
                            break;
                        default:
                            diagnosticSeverity = vscode.DiagnosticSeverity.Information;
                    }
                    diagnostics.push(new vscode.Diagnostic(range, message, diagnosticSeverity));
                }
            }
            diagnosticCollection.set(document.uri, diagnostics);
        }
        catch (error) {
            const diagnostics = [];
            // Parse stderr for linting errors or parser errors
            const errorOutput = error.stderr || error.stdout || "";
            const lines = errorOutput.split("\n");
            for (const line of lines) {
                if (!line.trim()) {
                    continue;
                }
                // Try to parse lint error format: file:line:column: severity: message
                let match = line.match(/^(.+?):(\d+):(\d+):\s+(error|warning|info):\s+(.+)$/);
                if (match) {
                    const [, , lineStr, colStr, severity, message] = match;
                    const lineNum = parseInt(lineStr, 10) - 1;
                    const colNum = parseInt(colStr, 10) - 1;
                    if (lineNum >= 0 && lineNum < document.lineCount) {
                        const range = new vscode.Range(lineNum, colNum, lineNum, Math.max(colNum + 1, document.lineAt(lineNum).text.length));
                        let diagnosticSeverity;
                        switch (severity) {
                            case "error":
                                diagnosticSeverity = vscode.DiagnosticSeverity.Error;
                                break;
                            case "warning":
                                diagnosticSeverity = vscode.DiagnosticSeverity.Warning;
                                break;
                            default:
                                diagnosticSeverity = vscode.DiagnosticSeverity.Information;
                        }
                        diagnostics.push(new vscode.Diagnostic(range, message, diagnosticSeverity));
                    }
                }
                else {
                    // Check for parser/lexer errors
                    const parserMatch = line.match(/(Parser error|Lexer error):\s+(.+)$/i);
                    if (parserMatch) {
                        // Show parser error on first line
                        const range = new vscode.Range(0, 0, 0, document.lineAt(0).text.length);
                        diagnostics.push(new vscode.Diagnostic(range, parserMatch[2], vscode.DiagnosticSeverity.Error));
                    }
                    else if (line.includes("Error:") || line.includes("error:")) {
                        // Generic error message - show on first line
                        const range = new vscode.Range(0, 0, 0, document.lineAt(0).text.length);
                        diagnostics.push(new vscode.Diagnostic(range, line, vscode.DiagnosticSeverity.Error));
                    }
                }
            }
            if (diagnostics.length > 0) {
                diagnosticCollection.set(document.uri, diagnostics);
            }
            else if (error.code === 127 || error.message?.includes("ENOENT")) {
                // Command not found
                const range = new vscode.Range(0, 0, 0, 0);
                diagnostics.push(new vscode.Diagnostic(range, `fip-lint not found at "${lintPath}". Please configure fipLinter.path in settings.`, vscode.DiagnosticSeverity.Warning));
                diagnosticCollection.set(document.uri, diagnostics);
            }
            else {
                // Log other errors for debugging
                console.error("FIP Linter error:", error);
            }
        }
    };
    // Lint on document open
    vscode.workspace.onDidOpenTextDocument(lintDocument, null, context.subscriptions);
    // Lint on document save
    vscode.workspace.onDidSaveTextDocument(lintDocument, null, context.subscriptions);
    // Lint on document change (debounced)
    let timeout;
    vscode.workspace.onDidChangeTextDocument((event) => {
        if (timeout) {
            clearTimeout(timeout);
        }
        timeout = setTimeout(() => {
            lintDocument(event.document);
        }, 500);
    }, null, context.subscriptions);
    // Lint all open FIP documents
    vscode.workspace.textDocuments.forEach((doc) => {
        if (doc.languageId === "fip") {
            lintDocument(doc);
        }
    });
}
exports.activate = activate;
function deactivate() { }
exports.deactivate = deactivate;
//# sourceMappingURL=extension.js.map