package com.kosac.ai.codegen.handlers;

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

import com.kosac.ai.codegen.Activator;
import com.kosac.ai.codegen.client.AgentClient;
import com.kosac.ai.codegen.client.AgentClientException;
import com.kosac.ai.codegen.model.GenerateRequest;
import com.kosac.ai.codegen.model.SpringGenerateResponse;
import com.kosac.ai.codegen.preferences.PreferenceConstants;

/**
 * Base handler for Spring backend code generation operations.
 *
 * This handler is intentionally "dumb" - it doesn't know anything about:
 * - LLM models, prompts, or configuration
 * - How the server generates code internally
 *
 * It only:
 * 1. Collects input from the user
 * 2. Sends request to the agent server with product="spring-backend"
 * 3. Creates files from the response (Controller, Service, DTO, Mapper)
 */
public abstract class AbstractSpringGenerateHandler extends AbstractHandler {

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

        // Set product to spring-backend
        request.setProduct("spring-backend");

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
     * Get the entity name from the request for file naming.
     *
     * @param request The generation request
     * @return The entity name (e.g., "Member")
     */
    protected abstract String getEntityName(GenerateRequest request);

    /**
     * Get the base package from preferences or default.
     */
    protected String getBasePackage() {
        String pkg = Activator.getDefault().getPreferenceStore()
                .getString(PreferenceConstants.P_SPRING_PACKAGE);
        if (pkg == null || pkg.isEmpty()) {
            return "com.company.project";
        }
        return pkg;
    }

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
        Job job = new Job("Generating Spring Backend Code") {
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
                    monitor.subTask("Generating Spring code (this may take a minute)...");
                    SpringGenerateResponse response = client.generateSpring(request);

                    if (!response.isSuccess()) {
                        showError(shell, "Generation failed: " + response.getError());
                        return Status.CANCEL_STATUS;
                    }

                    // Create files
                    monitor.subTask("Creating files...");
                    String entityName = getEntityName(request);
                    String basePackage = getBasePackage();
                    createFiles(project, entityName, basePackage, response);

                    // Show success with warnings if any
                    if (response.hasWarnings()) {
                        showWarning(shell, "Code generated with warnings:\n" +
                                String.join("\n", response.getWarnings()));
                    } else {
                        showInfo(shell, "Spring backend code generated successfully!\n\n" +
                                "Files created:\n" +
                                getCreatedFilesMessage(entityName));
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

    private String getCreatedFilesMessage(String entityName) {
        return "- controller/" + entityName + "Controller.java\n" +
               "- service/" + entityName + "Service.java\n" +
               "- service/impl/" + entityName + "ServiceImpl.java\n" +
               "- dto/" + entityName + "DTO.java\n" +
               "- mapper/" + entityName + "Mapper.java\n" +
               "- resources/mapper/" + entityName + "Mapper.xml";
    }

    private void createFiles(IProject project, String entityName, String basePackage,
                             SpringGenerateResponse response) throws CoreException {

        SpringGenerateResponse.Artifacts artifacts = response.getArtifacts();
        if (artifacts == null) {
            throw new CoreException(new Status(IStatus.ERROR, Activator.PLUGIN_ID,
                    "No artifacts in response"));
        }

        // Convert package to path
        String basePath = "src/main/java/" + basePackage.replace('.', '/');

        // Create Controller
        if (artifacts.getController() != null && !artifacts.getController().isEmpty()) {
            IFolder folder = ensureFolder(project, basePath + "/controller");
            IFile file = folder.getFile(entityName + "Controller.java");
            createOrUpdateFile(file, artifacts.getController());

            // Open in editor
            openFileInEditor(file);
        }

        // Create Service interface
        if (artifacts.getServiceInterface() != null && !artifacts.getServiceInterface().isEmpty()) {
            IFolder folder = ensureFolder(project, basePath + "/service");
            IFile file = folder.getFile(entityName + "Service.java");
            createOrUpdateFile(file, artifacts.getServiceInterface());
        }

        // Create Service implementation
        if (artifacts.getServiceImpl() != null && !artifacts.getServiceImpl().isEmpty()) {
            IFolder folder = ensureFolder(project, basePath + "/service/impl");
            IFile file = folder.getFile(entityName + "ServiceImpl.java");
            createOrUpdateFile(file, artifacts.getServiceImpl());
        }

        // Create DTO
        if (artifacts.getDto() != null && !artifacts.getDto().isEmpty()) {
            IFolder folder = ensureFolder(project, basePath + "/dto");
            IFile file = folder.getFile(entityName + "DTO.java");
            createOrUpdateFile(file, artifacts.getDto());
        }

        // Create Mapper interface
        if (artifacts.getMapperInterface() != null && !artifacts.getMapperInterface().isEmpty()) {
            IFolder folder = ensureFolder(project, basePath + "/mapper");
            IFile file = folder.getFile(entityName + "Mapper.java");
            createOrUpdateFile(file, artifacts.getMapperInterface());
        }

        // Create Mapper XML
        if (artifacts.getMapperXml() != null && !artifacts.getMapperXml().isEmpty()) {
            IFolder folder = ensureFolder(project, "src/main/resources/mapper");
            IFile file = folder.getFile(entityName + "Mapper.xml");
            createOrUpdateFile(file, artifacts.getMapperXml());
        }

        // Refresh project
        project.refreshLocal(IResource.DEPTH_INFINITE, null);
    }

    private IFolder ensureFolder(IProject project, String path) throws CoreException {
        IFolder folder = project.getFolder(path);
        if (!folder.exists()) {
            createFolderRecursive(folder);
        }
        return folder;
    }

    private void createFolderRecursive(IFolder folder) throws CoreException {
        IContainer parent = folder.getParent();
        if (parent instanceof IFolder && !parent.exists()) {
            createFolderRecursive((IFolder) parent);
        }
        if (!folder.exists()) {
            folder.create(true, true, null);
        }
    }

    private void createOrUpdateFile(IFile file, String content) throws CoreException {
        InputStream source = new ByteArrayInputStream(content.getBytes(StandardCharsets.UTF_8));

        if (file.exists()) {
            file.setContents(source, true, true, null);
        } else {
            file.create(source, true, null);
        }
    }

    private void openFileInEditor(IFile file) {
        Display.getDefault().asyncExec(() -> {
            try {
                IWorkbenchPage page = PlatformUI.getWorkbench()
                        .getActiveWorkbenchWindow().getActivePage();
                IDE.openEditor(page, file);
            } catch (PartInitException e) {
                // Ignore editor open errors
            }
        });
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
