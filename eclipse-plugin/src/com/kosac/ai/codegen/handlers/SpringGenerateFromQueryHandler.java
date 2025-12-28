package com.kosac.ai.codegen.handlers;

import org.eclipse.jface.window.Window;
import org.eclipse.swt.widgets.Shell;

import com.kosac.ai.codegen.dialogs.QueryInputDialog;
import com.kosac.ai.codegen.model.GenerateRequest;
import com.kosac.ai.codegen.model.QuerySampleInput;

/**
 * Handler for generating Spring backend code from query sample input.
 */
public class SpringGenerateFromQueryHandler extends AbstractSpringGenerateHandler {

    private QuerySampleInput lastQueryInput;

    @Override
    protected GenerateRequest showInputDialog(Shell shell) {
        QueryInputDialog dialog = new QueryInputDialog(shell);

        if (dialog.open() == Window.OK) {
            lastQueryInput = dialog.getQueryInput();
            GenerateRequest request = GenerateRequest.fromQuery(lastQueryInput);
            request.setProduct("spring-backend");
            request.getContext().setProject(getBasePackage());
            return request;
        }

        return null;
    }

    @Override
    protected String getEntityName(GenerateRequest request) {
        if (lastQueryInput != null) {
            return extractEntityNameFromQuery(lastQueryInput.getQuery());
        }
        return "Entity";
    }

    /**
     * Extract entity name from SQL query.
     */
    private String extractEntityNameFromQuery(String query) {
        String upper = query.toUpperCase();
        int fromPos = upper.indexOf(" FROM ");

        if (fromPos > 0) {
            String afterFrom = query.substring(fromPos + 6).trim();
            String[] parts = afterFrom.split("\\s+");
            if (parts.length > 0) {
                String tableName = parts[0].trim();
                // Remove schema prefix if present
                int dotPos = tableName.lastIndexOf('.');
                if (dotPos > 0) {
                    tableName = tableName.substring(dotPos + 1);
                }
                // Remove quotes
                tableName = tableName.replaceAll("[\"'`\\[\\]]", "");
                return tableToEntityName(tableName);
            }
        }

        return "Entity";
    }

    /**
     * Convert table name to entity name.
     */
    private String tableToEntityName(String tableName) {
        String clean = tableName;

        if (clean.startsWith("TB_")) {
            clean = clean.substring(3);
        } else if (clean.startsWith("TBL_")) {
            clean = clean.substring(4);
        } else if (clean.startsWith("T_")) {
            clean = clean.substring(2);
        }

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
