package com.kosac.ai.codegen.handlers;

import java.util.regex.Matcher;
import java.util.regex.Pattern;

import org.eclipse.jface.window.Window;
import org.eclipse.swt.widgets.Shell;

import com.kosac.ai.codegen.dialogs.QueryInputDialog;
import com.kosac.ai.codegen.model.GenerateRequest;
import com.kosac.ai.codegen.model.QuerySampleInput;

/**
 * Handler for generating code from SQL query sample.
 */
public class GenerateFromQueryHandler extends AbstractGenerateHandler {

    private static final Pattern FROM_PATTERN = Pattern.compile(
            "FROM\\s+(\\w+)", Pattern.CASE_INSENSITIVE);

    private QuerySampleInput lastQueryInput;

    @Override
    protected GenerateRequest showInputDialog(Shell shell) {
        QueryInputDialog dialog = new QueryInputDialog(shell);

        if (dialog.open() == Window.OK) {
            lastQueryInput = dialog.getQueryInput();
            return GenerateRequest.fromQuery(lastQueryInput);
        }

        return null;
    }

    @Override
    protected String getScreenName(GenerateRequest request) {
        if (lastQueryInput != null) {
            // Try to extract table name from query
            Matcher matcher = FROM_PATTERN.matcher(lastQueryInput.getQuery());
            if (matcher.find()) {
                return matcher.group(1).toLowerCase() + "_list";
            }
        }
        return "query_result";
    }
}
