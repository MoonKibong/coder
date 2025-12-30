package com.kosac.ai.codegen.dialogs;

import org.eclipse.jface.dialogs.TitleAreaDialog;
import org.eclipse.swt.SWT;
import org.eclipse.swt.layout.GridData;
import org.eclipse.swt.layout.GridLayout;
import org.eclipse.swt.widgets.Composite;
import org.eclipse.swt.widgets.Control;
import org.eclipse.swt.widgets.Label;
import org.eclipse.swt.widgets.Shell;
import org.eclipse.swt.widgets.Text;

import com.kosac.ai.codegen.model.QARequest;

/**
 * Dialog for inputting a question for Q&A.
 */
public class QADialog extends TitleAreaDialog {

    private Text questionText;
    private Text contextText;

    private String question;
    private String context;
    private String product;

    /**
     * Create dialog for specific product.
     * @param parentShell Parent shell
     * @param product Product identifier (xframe5-ui or spring-backend)
     */
    public QADialog(Shell parentShell, String product) {
        super(parentShell);
        this.product = product;
        setShellStyle(getShellStyle() | SWT.RESIZE);
    }

    @Override
    public void create() {
        super.create();
        setTitle("Ask a Question");
        if ("spring-backend".equals(product)) {
            setMessage("Ask a question about Spring Framework, Spring Boot, or Java development.");
        } else {
            setMessage("Ask a question about xFrame5 components, patterns, or development.");
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

        // Question input
        Label questionLabel = new Label(container, SWT.NONE);
        questionLabel.setText("Question:");
        questionLabel.setLayoutData(new GridData(SWT.LEFT, SWT.TOP, false, false));

        questionText = new Text(container, SWT.BORDER | SWT.MULTI | SWT.V_SCROLL | SWT.WRAP);
        GridData questionData = new GridData(SWT.FILL, SWT.FILL, true, true);
        questionData.heightHint = 100;
        questionData.widthHint = 500;
        questionText.setLayoutData(questionData);
        if ("spring-backend".equals(product)) {
            questionText.setMessage("e.g., How do I create a REST controller with pagination?");
        } else {
            questionText.setMessage("e.g., How do I use Dataset with Grid component?");
        }

        // Additional context
        Label contextLabel = new Label(container, SWT.NONE);
        contextLabel.setText("Context:");
        contextLabel.setLayoutData(new GridData(SWT.LEFT, SWT.TOP, false, false));

        contextText = new Text(container, SWT.BORDER | SWT.MULTI | SWT.V_SCROLL | SWT.WRAP);
        GridData contextData = new GridData(SWT.FILL, SWT.CENTER, true, false);
        contextData.heightHint = 60;
        contextText.setLayoutData(contextData);
        contextText.setMessage("Optional: Provide additional context about what you're trying to do...");

        // Examples
        Label exampleLabel = new Label(container, SWT.NONE);
        exampleLabel.setText("Examples:");
        exampleLabel.setLayoutData(new GridData(SWT.LEFT, SWT.TOP, false, false));

        Text exampleText = new Text(container, SWT.READ_ONLY | SWT.MULTI | SWT.WRAP);
        GridData exampleData = new GridData(SWT.FILL, SWT.CENTER, true, false);
        exampleData.heightHint = 80;
        exampleText.setLayoutData(exampleData);

        if ("spring-backend".equals(product)) {
            exampleText.setText(
                    "- How do I implement pagination in Spring Data JPA?\n" +
                    "- What's the best way to handle exceptions in REST controllers?\n" +
                    "- MyBatis에서 동적 쿼리를 어떻게 작성하나요?");
        } else {
            exampleText.setText(
                    "- How do I bind a Dataset to a Grid component?\n" +
                    "- What's the difference between on_click and on_itemdblclick?\n" +
                    "- xcomm.execute 함수를 어떻게 사용하나요?");
        }
        exampleText.setBackground(container.getDisplay().getSystemColor(SWT.COLOR_WIDGET_BACKGROUND));

        return area;
    }

    @Override
    protected void okPressed() {
        if (validateInput()) {
            buildQAInput();
            super.okPressed();
        }
    }

    private boolean validateInput() {
        String questionValue = questionText.getText().trim();
        if (questionValue.isEmpty()) {
            setErrorMessage("Please enter a question.");
            return false;
        }

        if (questionValue.length() < 5) {
            setErrorMessage("Please provide a more detailed question (at least 5 characters).");
            return false;
        }

        // Check for maximum size (5KB)
        if (questionValue.length() > 5 * 1024) {
            setErrorMessage("Question exceeds maximum size limit (5KB).");
            return false;
        }

        setErrorMessage(null);
        return true;
    }

    private void buildQAInput() {
        this.question = questionText.getText().trim();
        String contextValue = contextText.getText().trim();
        this.context = contextValue.isEmpty() ? null : contextValue;
    }

    public String getQuestion() {
        return question;
    }

    public String getContext() {
        return context;
    }

    public String getProduct() {
        return product;
    }

    /**
     * Build a QARequest from the dialog input.
     */
    public QARequest buildRequest() {
        QARequest request;
        if ("spring-backend".equals(product)) {
            request = QARequest.forSpring(question);
        } else {
            request = QARequest.forXFrame5(question);
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
