package com.softbase.xframe5.codegen.handlers;

import org.eclipse.jface.window.Window;
import org.eclipse.swt.widgets.Shell;

import com.softbase.xframe5.codegen.dialogs.SchemaInputDialog;
import com.softbase.xframe5.codegen.model.GenerateRequest;
import com.softbase.xframe5.codegen.model.SchemaInput;

/**
 * Handler for generating Spring backend code from DB schema input.
 */
public class SpringGenerateFromSchemaHandler extends AbstractSpringGenerateHandler {

    private SchemaInput lastSchemaInput;

    @Override
    protected GenerateRequest showInputDialog(Shell shell) {
        SchemaInputDialog dialog = new SchemaInputDialog(shell);

        if (dialog.open() == Window.OK) {
            lastSchemaInput = dialog.getSchemaInput();
            GenerateRequest request = GenerateRequest.fromSchema(lastSchemaInput);
            request.setProduct("spring-backend");
            request.getContext().setProject(getBasePackage());
            return request;
        }

        return null;
    }

    @Override
    protected String getEntityName(GenerateRequest request) {
        if (lastSchemaInput != null) {
            return tableToEntityName(lastSchemaInput.getTable());
        }
        return "Entity";
    }

    /**
     * Convert table name to entity name (e.g., "TB_MEMBER" -> "Member").
     */
    private String tableToEntityName(String tableName) {
        String clean = tableName;

        // Remove common prefixes
        if (clean.startsWith("TB_")) {
            clean = clean.substring(3);
        } else if (clean.startsWith("TBL_")) {
            clean = clean.substring(4);
        } else if (clean.startsWith("T_")) {
            clean = clean.substring(2);
        }

        // Convert to PascalCase
        StringBuilder result = new StringBuilder();
        boolean capitalizeNext = true;

        for (char c : clean.toCharArray()) {
            if (c == '_') {
                capitalizeNext = true;
            } else if (capitalizeNext) {
                result.append(Character.toUpperCase(c));
                capitalizeNext = false;
            } else {
                result.append(Character.toLowerCase(c));
            }
        }

        return result.toString();
    }
}
