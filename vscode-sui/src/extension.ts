import * as path from 'path';
import * as vscode from 'vscode';
import {
    LanguageClient,
    LanguageClientOptions,
    ServerOptions,
    TransportKind
} from 'vscode-languageclient/node';

let client: LanguageClient | undefined;

export async function activate(context: vscode.ExtensionContext) {
    const config = vscode.workspace.getConfiguration('sui.lsp');
    const enabled = config.get<boolean>('enabled', true);

    if (!enabled) {
        console.log('Sui language server is disabled');
        return;
    }

    const serverPath = config.get<string>('path', 'sui-lsp');

    // Server options - run the sui-lsp binary
    const serverOptions: ServerOptions = {
        run: {
            command: serverPath,
            transport: TransportKind.stdio
        },
        debug: {
            command: serverPath,
            transport: TransportKind.stdio
        }
    };

    // Client options
    const clientOptions: LanguageClientOptions = {
        documentSelector: [{ scheme: 'file', language: 'sui' }],
        synchronize: {
            fileEvents: vscode.workspace.createFileSystemWatcher('**/*.sui')
        },
        outputChannel: vscode.window.createOutputChannel('Sui Language Server')
    };

    // Create and start the client
    client = new LanguageClient(
        'sui-lsp',
        'Sui Language Server',
        serverOptions,
        clientOptions
    );

    try {
        await client.start();
        console.log('Sui language server started');
    } catch (error) {
        console.error('Failed to start Sui language server:', error);
        vscode.window.showWarningMessage(
            `Failed to start Sui language server. Make sure 'sui-lsp' is installed and in your PATH. ` +
            `Install with: cargo install sui-lang --features lsp`
        );
    }

    // Register commands
    context.subscriptions.push(
        vscode.commands.registerCommand('sui.restartServer', async () => {
            if (client) {
                await client.stop();
                await client.start();
                vscode.window.showInformationMessage('Sui language server restarted');
            }
        })
    );
}

export function deactivate(): Thenable<void> | undefined {
    if (!client) {
        return undefined;
    }
    return client.stop();
}
