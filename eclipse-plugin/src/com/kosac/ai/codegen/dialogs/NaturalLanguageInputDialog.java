package com.kosac.ai.codegen.dialogs;

import org.eclipse.jface.dialogs.TitleAreaDialog;
import org.eclipse.swt.SWT;
import org.eclipse.swt.layout.GridData;
import org.eclipse.swt.layout.GridLayout;
import org.eclipse.swt.widgets.Combo;
import org.eclipse.swt.widgets.Composite;
import org.eclipse.swt.widgets.Control;
import org.eclipse.swt.widgets.Label;
import org.eclipse.swt.widgets.Shell;
import org.eclipse.swt.widgets.Text;

import com.kosac.ai.codegen.model.NaturalLanguageInput;

/**
 * Dialog for inputting natural language description.
 */
public class NaturalLanguageInputDialog extends TitleAreaDialog {

    private Text descriptionText;
    private Combo screenTypeCombo;
    private Text contextText;

    private NaturalLanguageInput nlInput;

    private static final String[] SCREEN_TYPES = {
            "",  // Auto-detect
            "list",
            "detail",
            "popup",
            "list_with_popup"
    };

    private static final String[] SCREEN_TYPE_LABELS = {
            "(Auto-detect)",
            "List Screen",
            "Detail Screen",
            "Popup Screen",
            "List with Popup"
    };

    public NaturalLanguageInputDialog(Shell parentShell) {
        super(parentShell);
        setShellStyle(getShellStyle() | SWT.RESIZE);
    }

    @Override
    public void create() {
        super.create();
        setTitle("Generate from Description");
        setMessage("Describe the screen you want to generate in natural language.");
    }

    @Override
    protected Control createDialogArea(Composite parent) {
        Composite area = (Composite) super.createDialogArea(parent);

        Composite container = new Composite(area, SWT.NONE);
        container.setLayoutData(new GridData(SWT.FILL, SWT.FILL, true, true));
        GridLayout layout = new GridLayout(2, false);
        layout.marginHeight = 10;
        layout.marginWidth = 10;
        container.setLayout(layout);

        // Description
        Label descLabel = new Label(container, SWT.NONE);
        descLabel.setText("Description:");
        descLabel.setLayoutData(new GridData(SWT.LEFT, SWT.TOP, false, false));

        descriptionText = new Text(container, SWT.BORDER | SWT.MULTI | SWT.V_SCROLL | SWT.WRAP);
        GridData descData = new GridData(SWT.FILL, SWT.FILL, true, true);
        descData.heightHint = 100;
        descriptionText.setLayoutData(descData);
        descriptionText.setMessage("Describe the screen you want to create...");

        // Screen type
        Label typeLabel = new Label(container, SWT.NONE);
        typeLabel.setText("Screen Type:");

        screenTypeCombo = new Combo(container, SWT.READ_ONLY);
        screenTypeCombo.setLayoutData(new GridData(SWT.FILL, SWT.CENTER, true, false));
        screenTypeCombo.setItems(SCREEN_TYPE_LABELS);
        screenTypeCombo.select(0);

        // Additional context
        Label contextLabel = new Label(container, SWT.NONE);
        contextLabel.setText("Additional Context:");
        contextLabel.setLayoutData(new GridData(SWT.LEFT, SWT.TOP, false, false));

        contextText = new Text(container, SWT.BORDER | SWT.MULTI | SWT.V_SCROLL | SWT.WRAP);
        GridData contextData = new GridData(SWT.FILL, SWT.FILL, true, true);
        contextData.heightHint = 60;
        contextText.setLayoutData(contextData);
        contextText.setMessage("Optional: Provide additional context or requirements...");

        // Examples
        Label exampleLabel = new Label(container, SWT.NONE);
        exampleLabel.setText("Examples:");
        exampleLabel.setLayoutData(new GridData(SWT.LEFT, SWT.TOP, false, false));

        Text exampleText = new Text(container, SWT.READ_ONLY | SWT.MULTI | SWT.WRAP);
        GridData exampleData = new GridData(SWT.FILL, SWT.CENTER, true, false);
        exampleData.heightHint = 80;
        exampleText.setLayoutData(exampleData);
        exampleText.setText(
                "- Create a member list screen with search and delete functionality\n" +
                "- 회원 목록 화면을 만들어주세요. 검색, 추가, 삭제 기능이 필요합니다.\n" +
                "- Generate an order detail popup with customer and product information");
        exampleText.setBackground(container.getDisplay().getSystemColor(SWT.COLOR_WIDGET_BACKGROUND));

        return area;
    }

    @Override
    protected void okPressed() {
        if (validateInput()) {
            buildNLInput();
            super.okPressed();
        }
    }

    private boolean validateInput() {
        String description = descriptionText.getText().trim();
        if (description.isEmpty()) {
            setErrorMessage("Description is required.");
            return false;
        }

        if (description.length() < 10) {
            setErrorMessage("Please provide a more detailed description (at least 10 characters).");
            return false;
        }

        setErrorMessage(null);
        return true;
    }

    private void buildNLInput() {
        nlInput = new NaturalLanguageInput(descriptionText.getText().trim());

        int typeIndex = screenTypeCombo.getSelectionIndex();
        if (typeIndex > 0 && typeIndex < SCREEN_TYPES.length) {
            nlInput.setScreenType(SCREEN_TYPES[typeIndex]);
        }

        String context = contextText.getText().trim();
        if (!context.isEmpty()) {
            nlInput.setContext(context);
        }
    }

    public NaturalLanguageInput getNLInput() {
        return nlInput;
    }

    @Override
    protected boolean isResizable() {
        return true;
    }
}
