package com.kosac.ai.codegen.handlers;

import org.eclipse.core.commands.ExecutionEvent;
import org.eclipse.jface.window.Window;
import org.eclipse.swt.widgets.Shell;

import com.kosac.ai.codegen.dialogs.CodeReviewDialog;
import com.kosac.ai.codegen.model.ReviewRequest;

/**
 * Handler for reviewing code entered via dialog for Spring.
 * Menu: Spring > Review Code...
 */
public class SpringReviewCodeHandler extends AbstractReviewHandler {

    @Override
    protected String getProduct() {
        return "spring-backend";
    }

    @Override
    protected ReviewRequest getReviewRequest(Shell shell, ExecutionEvent event) {
        // Show code review dialog
        CodeReviewDialog dialog = new CodeReviewDialog(shell, getProduct());

        // Pre-populate file type from active editor extension
        String extension = getActiveFileExtension(event);
        if (extension != null) {
            dialog.create();
            dialog.setFileTypeFromExtension(extension);
        }

        if (dialog.open() == Window.OK) {
            return dialog.buildRequest();
        }

        return null; // User cancelled
    }
}
