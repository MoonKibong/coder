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
 * Handler for reviewing selected code in the editor for xFrame5.
 * Menu: xFrame5 > Review Selection
 */
public class XFrame5ReviewSelectionHandler extends AbstractReviewHandler {

    @Override
    protected String getProduct() {
        return "xframe5-ui";
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

        // Detect file type from extension
        String extension = getActiveFileExtension(event);
        String fileType = detectFileType(extension, selectedCode);

        // Build request
        ReviewRequest request = ReviewRequest.forXFrame5(selectedCode, fileType);
        return request;
    }

    private String detectFileType(String extension, String code) {
        if (extension != null) {
            String ext = extension.toLowerCase();
            if ("xml".equals(ext)) return "xml";
            if ("js".equals(ext)) return "javascript";
        }

        // Auto-detect from content
        if (code.trim().startsWith("<?xml") || code.trim().startsWith("<")) {
            return "xml";
        }
        if (code.contains("function ") || code.contains("var ") || code.contains("xcomm.")) {
            return "javascript";
        }

        return null; // Let server detect
    }
}
