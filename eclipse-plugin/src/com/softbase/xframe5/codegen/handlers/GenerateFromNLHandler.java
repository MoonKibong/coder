package com.softbase.xframe5.codegen.handlers;

import java.util.regex.Matcher;
import java.util.regex.Pattern;

import org.eclipse.jface.window.Window;
import org.eclipse.swt.widgets.Shell;

import com.softbase.xframe5.codegen.dialogs.NaturalLanguageInputDialog;
import com.softbase.xframe5.codegen.model.GenerateRequest;
import com.softbase.xframe5.codegen.model.NaturalLanguageInput;

/**
 * Handler for generating code from natural language description.
 */
public class GenerateFromNLHandler extends AbstractGenerateHandler {

    private static final Pattern ENTITY_PATTERN = Pattern.compile(
            "(member|user|order|product|customer|invoice|item|category)",
            Pattern.CASE_INSENSITIVE);

    private static final Pattern KOREAN_ENTITY_PATTERN = Pattern.compile(
            "(회원|사용자|주문|상품|제품|고객|송장|품목|카테고리)");

    private NaturalLanguageInput lastNLInput;

    @Override
    protected GenerateRequest showInputDialog(Shell shell) {
        NaturalLanguageInputDialog dialog = new NaturalLanguageInputDialog(shell);

        if (dialog.open() == Window.OK) {
            lastNLInput = dialog.getNLInput();
            return GenerateRequest.fromNaturalLanguage(lastNLInput);
        }

        return null;
    }

    @Override
    protected String getScreenName(GenerateRequest request) {
        if (lastNLInput != null) {
            String desc = lastNLInput.getDescription();

            // Try to extract entity name from description
            Matcher matcher = ENTITY_PATTERN.matcher(desc);
            if (matcher.find()) {
                String entity = matcher.group(1).toLowerCase();
                String screenType = getScreenTypeSuffix(lastNLInput.getScreenType());
                return entity + screenType;
            }

            // Try Korean entity names
            Matcher koreanMatcher = KOREAN_ENTITY_PATTERN.matcher(desc);
            if (koreanMatcher.find()) {
                String koreanEntity = koreanMatcher.group(1);
                String englishEntity = translateKoreanEntity(koreanEntity);
                String screenType = getScreenTypeSuffix(lastNLInput.getScreenType());
                return englishEntity + screenType;
            }
        }

        return "nl_generated_screen";
    }

    private String getScreenTypeSuffix(String screenType) {
        if (screenType == null || screenType.isEmpty()) {
            return "_list"; // default
        }

        switch (screenType.toLowerCase()) {
            case "list":
                return "_list";
            case "detail":
                return "_detail";
            case "popup":
                return "_popup";
            case "list_with_popup":
                return "_list";
            default:
                return "_list";
        }
    }

    private String translateKoreanEntity(String korean) {
        switch (korean) {
            case "회원":
            case "사용자":
                return "member";
            case "주문":
                return "order";
            case "상품":
            case "제품":
                return "product";
            case "고객":
                return "customer";
            case "송장":
                return "invoice";
            case "품목":
                return "item";
            case "카테고리":
                return "category";
            default:
                return "entity";
        }
    }
}
