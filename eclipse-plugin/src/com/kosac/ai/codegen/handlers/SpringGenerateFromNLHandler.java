package com.kosac.ai.codegen.handlers;

import org.eclipse.jface.window.Window;
import org.eclipse.swt.widgets.Shell;

import com.kosac.ai.codegen.dialogs.NaturalLanguageInputDialog;
import com.kosac.ai.codegen.model.GenerateRequest;
import com.kosac.ai.codegen.model.NaturalLanguageInput;

/**
 * Handler for generating Spring backend code from natural language description.
 */
public class SpringGenerateFromNLHandler extends AbstractSpringGenerateHandler {

    private NaturalLanguageInput lastNLInput;

    @Override
    protected GenerateRequest showInputDialog(Shell shell) {
        NaturalLanguageInputDialog dialog = new NaturalLanguageInputDialog(shell);

        if (dialog.open() == Window.OK) {
            lastNLInput = dialog.getNLInput();
            GenerateRequest request = GenerateRequest.fromNaturalLanguage(lastNLInput);
            request.setProduct("spring-backend");
            request.getContext().setProject(getBasePackage());
            return request;
        }

        return null;
    }

    @Override
    protected String getEntityName(GenerateRequest request) {
        if (lastNLInput != null) {
            return inferEntityName(lastNLInput.getDescription());
        }
        return "Entity";
    }

    /**
     * Infer entity name from natural language description.
     */
    private String inferEntityName(String description) {
        String lower = description.toLowerCase();

        // Common patterns
        if (lower.contains("회원") || lower.contains("member") || lower.contains("user")) {
            return "Member";
        }
        if (lower.contains("주문") || lower.contains("order")) {
            return "Order";
        }
        if (lower.contains("상품") || lower.contains("product") || lower.contains("item")) {
            return "Product";
        }
        if (lower.contains("게시판") || lower.contains("board") || lower.contains("post")) {
            return "Board";
        }
        if (lower.contains("고객") || lower.contains("customer")) {
            return "Customer";
        }
        if (lower.contains("직원") || lower.contains("employee")) {
            return "Employee";
        }
        if (lower.contains("부서") || lower.contains("department")) {
            return "Department";
        }
        if (lower.contains("코드") || lower.contains("code")) {
            return "Code";
        }

        return "Entity";
    }
}
