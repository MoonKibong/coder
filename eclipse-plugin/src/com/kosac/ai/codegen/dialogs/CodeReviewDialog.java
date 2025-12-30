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

import com.kosac.ai.codegen.model.ReviewRequest;

/**
 * Dialog for inputting code for review.
 * Allows users to paste code and select file type.
 */
public class CodeReviewDialog extends TitleAreaDialog {

    private Text codeText;
    private Combo fileTypeCombo;
    private Text contextText;

    private String code;
    private String fileType;
    private String context;
    private String product;

    private static final String[] FILE_TYPES = {
            "",  // Auto-detect
            "xml",
            "javascript",
            "java"
    };

    private static final String[] FILE_TYPE_LABELS = {
            "(Auto-detect)",
            "XML (xFrame5 Screen)",
            "JavaScript (xFrame5 Script)",
            "Java (Spring Controller/Service)"
    };

    /**
     * Create dialog for specific product.
     * @param parentShell Parent shell
     * @param product Product identifier (xframe5-ui or spring-backend)
     */
    public CodeReviewDialog(Shell parentShell, String product) {
        super(parentShell);
        this.product = product;
        setShellStyle(getShellStyle() | SWT.RESIZE | SWT.MAX);
    }

    @Override
    public void create() {
        super.create();
        setTitle("Code Review");
        if ("spring-backend".equals(product)) {
            setMessage("Paste your Spring Java code for AI-powered review.");
        } else {
            setMessage("Paste your xFrame5 XML or JavaScript code for AI-powered review.");
        }
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

        // File Type
        Label typeLabel = new Label(container, SWT.NONE);
        typeLabel.setText("File Type:");

        fileTypeCombo = new Combo(container, SWT.READ_ONLY);
        fileTypeCombo.setLayoutData(new GridData(SWT.FILL, SWT.CENTER, true, false));
        if ("spring-backend".equals(product)) {
            fileTypeCombo.setItems(new String[]{"(Auto-detect)", "Java (Spring Controller/Service)"});
        } else {
            fileTypeCombo.setItems(FILE_TYPE_LABELS);
        }
        fileTypeCombo.select(0);

        // Code input
        Label codeLabel = new Label(container, SWT.NONE);
        codeLabel.setText("Code:");
        codeLabel.setLayoutData(new GridData(SWT.LEFT, SWT.TOP, false, false));

        codeText = new Text(container, SWT.BORDER | SWT.MULTI | SWT.V_SCROLL | SWT.H_SCROLL);
        GridData codeData = new GridData(SWT.FILL, SWT.FILL, true, true);
        codeData.heightHint = 300;
        codeData.widthHint = 600;
        codeText.setLayoutData(codeData);
        codeText.setMessage("Paste your code here for review...");

        // Additional context
        Label contextLabel = new Label(container, SWT.NONE);
        contextLabel.setText("Context:");
        contextLabel.setLayoutData(new GridData(SWT.LEFT, SWT.TOP, false, false));

        contextText = new Text(container, SWT.BORDER | SWT.MULTI | SWT.V_SCROLL | SWT.WRAP);
        GridData contextData = new GridData(SWT.FILL, SWT.CENTER, true, false);
        contextData.heightHint = 60;
        contextText.setLayoutData(contextData);
        contextText.setMessage("Optional: Describe what this code does or specific areas to focus on...");

        // Info label
        Label infoLabel = new Label(container, SWT.NONE);
        infoLabel.setText("Info:");
        infoLabel.setLayoutData(new GridData(SWT.LEFT, SWT.TOP, false, false));

        Text infoText = new Text(container, SWT.READ_ONLY | SWT.MULTI | SWT.WRAP);
        GridData infoData = new GridData(SWT.FILL, SWT.CENTER, true, false);
        infoData.heightHint = 60;
        infoText.setLayoutData(infoData);
        infoText.setText(
                "The AI will review your code for:\n" +
                "- Syntax issues and errors\n" +
                "- Pattern and naming convention violations\n" +
                "- Performance and security concerns\n" +
                "- Best practice recommendations");
        infoText.setBackground(container.getDisplay().getSystemColor(SWT.COLOR_WIDGET_BACKGROUND));

        return area;
    }

    @Override
    protected void okPressed() {
        if (validateInput()) {
            buildReviewInput();
            super.okPressed();
        }
    }

    private boolean validateInput() {
        String codeValue = codeText.getText().trim();
        if (codeValue.isEmpty()) {
            setErrorMessage("Code is required for review.");
            return false;
        }

        if (codeValue.length() < 10) {
            setErrorMessage("Please provide more code (at least 10 characters).");
            return false;
        }

        // Check for maximum size (50KB)
        if (codeValue.length() > 50 * 1024) {
            setErrorMessage("Code exceeds maximum size limit (50KB). Please reduce the code size.");
            return false;
        }

        setErrorMessage(null);
        return true;
    }

    private void buildReviewInput() {
        this.code = codeText.getText();

        int typeIndex = fileTypeCombo.getSelectionIndex();
        if ("spring-backend".equals(product)) {
            // For Spring, only Java is an option
            this.fileType = typeIndex > 0 ? "java" : null;
        } else {
            if (typeIndex > 0 && typeIndex < FILE_TYPES.length) {
                this.fileType = FILE_TYPES[typeIndex];
            } else {
                this.fileType = null; // Auto-detect
            }
        }

        String contextValue = contextText.getText().trim();
        this.context = contextValue.isEmpty() ? null : contextValue;
    }

    /**
     * Set initial code (for pre-populating from editor selection).
     */
    public void setCode(String code) {
        if (codeText != null) {
            codeText.setText(code != null ? code : "");
        }
    }

    /**
     * Set file type based on file extension.
     */
    public void setFileTypeFromExtension(String extension) {
        if (extension == null || fileTypeCombo == null) return;

        String ext = extension.toLowerCase();
        int index = 0;
        if ("xml".equals(ext)) {
            index = 1;
        } else if ("js".equals(ext)) {
            index = 2;
        } else if ("java".equals(ext)) {
            index = "spring-backend".equals(product) ? 1 : 3;
        }
        fileTypeCombo.select(index);
    }

    public String getCode() {
        return code;
    }

    public String getFileType() {
        return fileType;
    }

    public String getContext() {
        return context;
    }

    public String getProduct() {
        return product;
    }

    /**
     * Build a ReviewRequest from the dialog input.
     */
    public ReviewRequest buildRequest() {
        ReviewRequest request;
        if ("spring-backend".equals(product)) {
            request = ReviewRequest.forSpring(code, fileType);
        } else {
            request = ReviewRequest.forXFrame5(code, fileType);
        }

        if (context != null) {
            request.getInput().setContext(context);
        }

        return request;
    }

    @Override
    protected boolean isResizable() {
        return true;
    }
}
