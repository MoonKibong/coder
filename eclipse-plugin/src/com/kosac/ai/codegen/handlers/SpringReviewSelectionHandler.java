package com.kosac.ai.codegen.handlers;

import org.eclipse.core.commands.ExecutionEvent;
import org.eclipse.jface.dialogs.MessageDialog;
import org.eclipse.jface.text.ITextSelection;
import org.eclipse.jface.viewers.ISelection;
import org.eclipse.swt.widgets.Shell;
import org.eclipse.ui.IEditorPart;
import org.eclipse.ui.handlers.HandlerUtil;
import org.eclipse.ui.texteditor.ITextEditor;

import com.kosac.ai.codegen.model.ReviewRequest;

/**
 * Handler for reviewing selected code in the editor for Spring.
 * Menu: Spring > Review Selection
 */
public class SpringReviewSelectionHandler extends AbstractReviewHandler {

    @Override
    protected String getProduct() {
        return "spring-backend";
    }

    @Override
    protected ReviewRequest getReviewRequest(Shell shell, ExecutionEvent event) {
        // Get selected text from editor
        IEditorPart editor = HandlerUtil.getActiveEditor(event);
        if (editor == null) {
            MessageDialog.openError(shell, "Error", "No active editor found.");
            return null;
        }

        if (!(editor instanceof ITextEditor)) {
            MessageDialog.openError(shell, "Error", "Please use a text editor.");
            return null;
        }

        ITextEditor textEditor = (ITextEditor) editor;
        ISelection selection = textEditor.getSelectionProvider().getSelection();

        if (!(selection instanceof ITextSelection)) {
            MessageDialog.openError(shell, "Error", "Please select some code to review.");
            return null;
        }

        ITextSelection textSelection = (ITextSelection) selection;
        String selectedCode = textSelection.getText();

        if (selectedCode == null || selectedCode.trim().isEmpty()) {
            MessageDialog.openError(shell, "Error", "Please select some code to review.");
            return null;
        }

        // Check minimum code size
        if (selectedCode.trim().length() < 10) {
            MessageDialog.openError(shell, "Error", "Please select more code (at least 10 characters).");
            return null;
        }

        // For Spring, file type is always Java
        String fileType = "java";

        // Build request
        ReviewRequest request = ReviewRequest.forSpring(selectedCode, fileType);
        return request;
    }
}
