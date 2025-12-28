package com.softbase.xframe5.codegen.handlers;

import java.io.ByteArrayInputStream;
import java.io.InputStream;
import java.nio.charset.StandardCharsets;

import org.eclipse.core.commands.AbstractHandler;
import org.eclipse.core.commands.ExecutionEvent;
import org.eclipse.core.commands.ExecutionException;
import org.eclipse.core.resources.IContainer;
import org.eclipse.core.resources.IFile;
import org.eclipse.core.resources.IFolder;
import org.eclipse.core.resources.IProject;
import org.eclipse.core.resources.IResource;
import org.eclipse.core.runtime.CoreException;
import org.eclipse.core.runtime.IProgressMonitor;
import org.eclipse.core.runtime.IStatus;
import org.eclipse.core.runtime.Status;
import org.eclipse.core.runtime.jobs.Job;
import org.eclipse.jface.dialogs.MessageDialog;
import org.eclipse.jface.viewers.ISelection;
import org.eclipse.jface.viewers.IStructuredSelection;
import org.eclipse.swt.widgets.Display;
import org.eclipse.swt.widgets.Shell;
import org.eclipse.ui.IWorkbenchPage;
import org.eclipse.ui.IWorkbenchWindow;
import org.eclipse.ui.PartInitException;
import org.eclipse.ui.PlatformUI;
import org.eclipse.ui.handlers.HandlerUtil;
import org.eclipse.ui.ide.IDE;

import com.softbase.xframe5.codegen.Activator;
import com.softbase.xframe5.codegen.client.AgentClient;
import com.softbase.xframe5.codegen.client.AgentClientException;
import com.softbase.xframe5.codegen.model.GenerateRequest;
import com.softbase.xframe5.codegen.model.GenerateResponse;
import com.softbase.xframe5.codegen.preferences.PreferenceConstants;

/**
 * Base handler for code generation operations.
 *
 * This handler is intentionally "dumb" - it doesn't know anything about:
 * - LLM models, prompts, or configuration
 * - How the server generates code internally
 *
 * It only:
 * 1. Collects input from the user
 * 2. Sends request to the agent server
 * 3. Creates files from the response
 */
public abstract class AbstractGenerateHandler extends AbstractHandler {

    @Override
    public Object execute(ExecutionEvent event) throws ExecutionException {
        Shell shell = HandlerUtil.getActiveShell(event);
        IProject project = getSelectedProject(event);

        if (project == null) {
            MessageDialog.openError(shell, "Error", "Please select a project first.");
            return null;
        }

        // Show input dialog and get request
        GenerateRequest request = showInputDialog(shell);
        if (request == null) {
            return null; // User cancelled
        }

        // Set project context
        request.getContext().setProject(project.getName());

        // Run generation in background
        runGeneration(shell, project, request);

        return null;
    }

    /**
     * Show the input dialog and return the request.
     * Subclasses must implement this to show the appropriate dialog.
     *
     * @param shell The parent shell
     * @return The generation request, or null if cancelled
     */
    protected abstract GenerateRequest showInputDialog(Shell shell);

    /**
     * Get the screen name from the request for file naming.
     *
     * @param request The generation request
     * @return The screen name
     */
    protected abstract String getScreenName(GenerateRequest request);

    private IProject getSelectedProject(ExecutionEvent event) {
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

    private void runGeneration(Shell shell, IProject project, GenerateRequest request) {
        Job job = new Job("Generating xFrame5 Code") {
            @Override
            protected IStatus run(IProgressMonitor monitor) {
                try {
                    monitor.beginTask("Generating code...", IProgressMonitor.UNKNOWN);

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

                    // Generate code
                    monitor.subTask("Generating code (this may take a minute)...");
                    GenerateResponse response = client.generate(request);

                    if (!response.isSuccess()) {
                        showError(shell, "Generation failed: " + response.getError());
                        return Status.CANCEL_STATUS;
                    }

                    // Create files
                    monitor.subTask("Creating files...");
                    String screenName = getScreenName(request);
                    createFiles(project, screenName, response);

                    // Show success with warnings if any
                    if (response.hasWarnings()) {
                        showWarning(shell, "Code generated with warnings:\n" +
                                String.join("\n", response.getWarnings()));
                    } else {
                        showInfo(shell, "Code generated successfully!\n\n" +
                                "Files created:\n" +
                                "- views/" + screenName + ".xml\n" +
                                "- scripts/" + screenName + ".js");
                    }

                    return Status.OK_STATUS;

                } catch (AgentClientException e) {
                    showError(shell, "Generation failed: " + e.getMessage());
                    return Status.CANCEL_STATUS;
                } catch (CoreException e) {
                    showError(shell, "Failed to create files: " + e.getMessage());
                    return Status.CANCEL_STATUS;
                } finally {
                    monitor.done();
                }
            }
        };

        job.setUser(true);
        job.schedule();
    }

    private void createFiles(IProject project, String screenName, GenerateResponse response)
            throws CoreException {

        GenerateResponse.Artifacts artifacts = response.getArtifacts();
        if (artifacts == null) {
            throw new CoreException(new Status(IStatus.ERROR, Activator.PLUGIN_ID,
                    "No artifacts in response"));
        }

        // Create views folder if needed
        IFolder viewsFolder = project.getFolder("views");
        if (!viewsFolder.exists()) {
            createFolder(viewsFolder);
        }

        // Create scripts folder if needed
        IFolder scriptsFolder = project.getFolder("scripts");
        if (!scriptsFolder.exists()) {
            createFolder(scriptsFolder);
        }

        // Create XML file
        if (artifacts.getXml() != null && !artifacts.getXml().isEmpty()) {
            IFile xmlFile = viewsFolder.getFile(screenName + ".xml");
            createOrUpdateFile(xmlFile, artifacts.getXml());

            // Open in editor
            Display.getDefault().asyncExec(() -> {
                try {
                    IWorkbenchPage page = PlatformUI.getWorkbench()
                            .getActiveWorkbenchWindow().getActivePage();
                    IDE.openEditor(page, xmlFile);
                } catch (PartInitException e) {
                    // Ignore editor open errors
                }
            });
        }

        // Create JavaScript file
        if (artifacts.getJavascript() != null && !artifacts.getJavascript().isEmpty()) {
            IFile jsFile = scriptsFolder.getFile(screenName + ".js");
            createOrUpdateFile(jsFile, artifacts.getJavascript());
        }

        // Refresh project
        project.refreshLocal(IResource.DEPTH_INFINITE, null);
    }

    private void createFolder(IFolder folder) throws CoreException {
        IContainer parent = folder.getParent();
        if (parent instanceof IFolder && !parent.exists()) {
            createFolder((IFolder) parent);
        }
        folder.create(true, true, null);
    }

    private void createOrUpdateFile(IFile file, String content) throws CoreException {
        InputStream source = new ByteArrayInputStream(content.getBytes(StandardCharsets.UTF_8));

        if (file.exists()) {
            file.setContents(source, true, true, null);
        } else {
            file.create(source, true, null);
        }
    }

    private void showError(Shell shell, String message) {
        Display.getDefault().asyncExec(() ->
                MessageDialog.openError(shell, "Generation Error", message));
    }

    private void showWarning(Shell shell, String message) {
        Display.getDefault().asyncExec(() ->
                MessageDialog.openWarning(shell, "Generation Complete", message));
    }

    private void showInfo(Shell shell, String message) {
        Display.getDefault().asyncExec(() ->
                MessageDialog.openInformation(shell, "Generation Complete", message));
    }
}
