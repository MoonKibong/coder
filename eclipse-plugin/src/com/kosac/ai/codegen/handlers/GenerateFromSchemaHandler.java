package com.kosac.ai.codegen.handlers;

import org.eclipse.jface.window.Window;
import org.eclipse.swt.widgets.Shell;

import com.kosac.ai.codegen.dialogs.SchemaInputDialog;
import com.kosac.ai.codegen.model.GenerateRequest;
import com.kosac.ai.codegen.model.SchemaInput;

/**
 * Handler for generating code from DB schema input.
 */
public class GenerateFromSchemaHandler extends AbstractGenerateHandler {

    private SchemaInput lastSchemaInput;

    @Override
    protected GenerateRequest showInputDialog(Shell shell) {
        SchemaInputDialog dialog = new SchemaInputDialog(shell);

        if (dialog.open() == Window.OK) {
            lastSchemaInput = dialog.getSchemaInput();
            return GenerateRequest.fromSchema(lastSchemaInput);
        }

        return null;
    }

    @Override
    protected String getScreenName(GenerateRequest request) {
        if (lastSchemaInput != null) {
            return lastSchemaInput.getTable().toLowerCase() + "_list";
        }
        return "generated_screen";
    }
}
