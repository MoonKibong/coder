package com.kosac.ai.codegen.handlers;

import org.eclipse.core.commands.AbstractHandler;
import org.eclipse.core.commands.ExecutionEvent;
import org.eclipse.core.commands.ExecutionException;
import org.eclipse.core.runtime.IProgressMonitor;
import org.eclipse.core.runtime.IStatus;
import org.eclipse.core.runtime.Status;
import org.eclipse.core.runtime.jobs.Job;
import org.eclipse.jface.dialogs.MessageDialog;
import org.eclipse.jface.window.Window;
import org.eclipse.swt.widgets.Display;
import org.eclipse.swt.widgets.Shell;
import org.eclipse.ui.handlers.HandlerUtil;

import com.kosac.ai.codegen.Activator;
import com.kosac.ai.codegen.client.AgentClient;
import com.kosac.ai.codegen.client.AgentClientException;
import com.kosac.ai.codegen.dialogs.QADialog;
import com.kosac.ai.codegen.dialogs.QAResultDialog;
import com.kosac.ai.codegen.model.QARequest;
import com.kosac.ai.codegen.model.QAResponse;
import com.kosac.ai.codegen.preferences.PreferenceConstants;

/**
 * Handler for xFrame5 Q&A.
 * Menu: xFrame5 > Ask Question...
 */
public class XFrame5QAHandler extends AbstractHandler {

    private static final String PRODUCT = "xframe5-ui";

    @Override
    public Object execute(ExecutionEvent event) throws ExecutionException {
        Shell shell = HandlerUtil.getActiveShell(event);

        // Show Q&A dialog
        QADialog dialog = new QADialog(shell, PRODUCT);
        if (dialog.open() != Window.OK) {
            return null; // User cancelled
        }

        QARequest request = dialog.buildRequest();

        // Run Q&A in background
        runQA(shell, request);

        return null;
    }

    private void runQA(Shell shell, QARequest request) {
        Job job = new Job("Getting Answer") {
            @Override
            protected IStatus run(IProgressMonitor monitor) {
                try {
                    monitor.beginTask("Getting answer...", IProgressMonitor.UNKNOWN);

                    // Get server endpoint from preferences
                    String endpoint = Activator.getDefault().getPreferenceStore()
                            .getString(PreferenceConstants.P_SERVER_ENDPOINT);

                    if (endpoint == null || endpoint.isEmpty()) {
                        endpoint = PreferenceConstants.DEFAULT_SERVER_ENDPOINT;
                    }

                    AgentClient client = new AgentClient(endpoint);

                    // Check server health first
                    monitor.subTask("Checking server availability...");
                    try {
                        if (!client.healthCheck()) {
                            showError(shell, "Server is not healthy. Please check your configuration.");
                            return Status.CANCEL_STATUS;
                        }
                    } catch (AgentClientException e) {
                        showError(shell, "Cannot connect to agent server: " + e.getMessage());
                        return Status.CANCEL_STATUS;
                    }

                    // Get answer
                    monitor.subTask("Searching knowledge base and generating answer...");
                    QAResponse response = client.qa(request);

                    // Show results
                    showResults(shell, response);

                    return Status.OK_STATUS;

                } catch (AgentClientException e) {
                    showError(shell, "Q&A failed: " + e.getMessage());
                    return Status.CANCEL_STATUS;
                } finally {
                    monitor.done();
                }
            }
        };

        job.setUser(true);
        job.schedule();
    }

    private void showResults(Shell shell, QAResponse response) {
        Display.getDefault().asyncExec(() -> {
            QAResultDialog resultDialog = new QAResultDialog(shell, response);
            resultDialog.open();
        });
    }

    private void showError(Shell shell, String message) {
        Display.getDefault().asyncExec(() ->
                MessageDialog.openError(shell, "Q&A Error", message));
    }
}
