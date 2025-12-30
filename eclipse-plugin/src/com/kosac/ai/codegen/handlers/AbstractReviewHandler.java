package com.kosac.ai.codegen.handlers;

import org.eclipse.core.commands.AbstractHandler;
import org.eclipse.core.commands.ExecutionEvent;
import org.eclipse.core.commands.ExecutionException;
import org.eclipse.core.resources.IFile;
import org.eclipse.core.resources.IProject;
import org.eclipse.core.resources.IResource;
import org.eclipse.core.runtime.IProgressMonitor;
import org.eclipse.core.runtime.IStatus;
import org.eclipse.core.runtime.Status;
import org.eclipse.core.runtime.jobs.Job;
import org.eclipse.jface.dialogs.MessageDialog;
import org.eclipse.jface.viewers.ISelection;
import org.eclipse.jface.viewers.IStructuredSelection;
import org.eclipse.jface.window.Window;
import org.eclipse.swt.widgets.Display;
import org.eclipse.swt.widgets.Shell;
import org.eclipse.ui.IWorkbenchPage;
import org.eclipse.ui.IWorkbenchWindow;
import org.eclipse.ui.handlers.HandlerUtil;

import com.kosac.ai.codegen.Activator;
import com.kosac.ai.codegen.client.AgentClient;
import com.kosac.ai.codegen.client.AgentClientException;
import com.kosac.ai.codegen.dialogs.CodeReviewResultDialog;
import com.kosac.ai.codegen.model.CodeReviewResponse;
import com.kosac.ai.codegen.model.ReviewRequest;
import com.kosac.ai.codegen.preferences.PreferenceConstants;

/**
 * Base handler for code review operations.
 *
 * This handler is intentionally "dumb" - it doesn't know anything about:
 * - LLM models, prompts, or configuration
 * - How the server performs code review internally
 *
 * It only:
 * 1. Collects code input from the user
 * 2. Sends request to the agent server
 * 3. Displays the review results
 */
public abstract class AbstractReviewHandler extends AbstractHandler {

    /**
     * Get the product identifier for this handler.
     * @return "xframe5-ui" or "spring-backend"
     */
    protected abstract String getProduct();

    /**
     * Get the code to review.
     * Subclasses implement this to either show a dialog or get from editor selection.
     *
     * @param shell The parent shell
     * @param event The execution event
     * @return The ReviewRequest, or null if cancelled
     */
    protected abstract ReviewRequest getReviewRequest(Shell shell, ExecutionEvent event);

    @Override
    public Object execute(ExecutionEvent event) throws ExecutionException {
        Shell shell = HandlerUtil.getActiveShell(event);
        IProject project = getSelectedProject(event);

        // Get the review request (code + options)
        ReviewRequest request = getReviewRequest(shell, event);
        if (request == null) {
            return null; // User cancelled
        }

        // Set project context if available
        if (project != null) {
            request.getContext().setProject(project.getName());
        }

        // Set filename from active editor if available
        String fileName = getActiveFileName(event);
        if (fileName != null) {
            request.getContext().setFileName(fileName);
        }

        // Run review in background
        runReview(shell, request);

        return null;
    }

    protected IProject getSelectedProject(ExecutionEvent event) {
        ISelection selection = HandlerUtil.getCurrentSelection(event);

        if (selection instanceof IStructuredSelection) {
            Object firstElement = ((IStructuredSelection) selection).getFirstElement();

            if (firstElement instanceof IResource) {
                return ((IResource) firstElement).getProject();
            }
        }

        // Try to get from active editor
        IWorkbenchWindow window = HandlerUtil.getActiveWorkbenchWindow(event);
        if (window != null) {
            IWorkbenchPage page = window.getActivePage();
            if (page != null && page.getActiveEditor() != null) {
                IFile file = page.getActiveEditor().getEditorInput().getAdapter(IFile.class);
                if (file != null) {
                    return file.getProject();
                }
            }
        }

        return null;
    }

    protected String getActiveFileName(ExecutionEvent event) {
        IWorkbenchWindow window = HandlerUtil.getActiveWorkbenchWindow(event);
        if (window != null) {
            IWorkbenchPage page = window.getActivePage();
            if (page != null && page.getActiveEditor() != null) {
                IFile file = page.getActiveEditor().getEditorInput().getAdapter(IFile.class);
                if (file != null) {
                    return file.getName();
                }
            }
        }
        return null;
    }

    protected String getActiveFileExtension(ExecutionEvent event) {
        String fileName = getActiveFileName(event);
        if (fileName != null && fileName.contains(".")) {
            return fileName.substring(fileName.lastIndexOf('.') + 1);
        }
        return null;
    }

    private void runReview(Shell shell, ReviewRequest request) {
        Job job = new Job("Reviewing Code") {
            @Override
            protected IStatus run(IProgressMonitor monitor) {
                try {
                    monitor.beginTask("Reviewing code...", IProgressMonitor.UNKNOWN);

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

                    // Review code
                    monitor.subTask("Analyzing code (this may take a minute)...");
                    CodeReviewResponse response = client.review(request);

                    // Show results
                    showResults(shell, response);

                    return Status.OK_STATUS;

                } catch (AgentClientException e) {
                    showError(shell, "Review failed: " + e.getMessage());
                    return Status.CANCEL_STATUS;
                } finally {
                    monitor.done();
                }
            }
        };

        job.setUser(true);
        job.schedule();
    }

    private void showResults(Shell shell, CodeReviewResponse response) {
        Display.getDefault().asyncExec(() -> {
            CodeReviewResultDialog resultDialog = new CodeReviewResultDialog(shell, response);
            resultDialog.open();
        });
    }

    protected void showError(Shell shell, String message) {
        Display.getDefault().asyncExec(() ->
                MessageDialog.openError(shell, "Review Error", message));
    }

    protected void showInfo(Shell shell, String message) {
        Display.getDefault().asyncExec(() ->
                MessageDialog.openInformation(shell, "Review", message));
    }
}
