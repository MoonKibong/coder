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

import com.kosac.ai.codegen.model.QAResponse;
import com.kosac.ai.codegen.model.QAResponse.CodeExample;
import com.kosac.ai.codegen.model.QAResponse.KnowledgeReference;
import com.kosac.ai.codegen.model.QAResponse.QAAnswer;

/**
 * Dialog for displaying Q&A results.
 */
public class QAResultDialog extends TitleAreaDialog {

    private QAResponse response;

    public QAResultDialog(Shell parentShell, QAResponse response) {
        super(parentShell);
        this.response = response;
        setShellStyle(getShellStyle() | SWT.RESIZE | SWT.MAX);
    }

    @Override
    public void create() {
        super.create();
        setTitle("Answer");

        if (response.isSuccess() && response.getAnswer() != null) {
            if (response.hasReferences()) {
                setMessage(String.format("Found %d relevant knowledge references",
                    response.getReferences().size()));
            } else {
                setMessage("Answer generated successfully");
            }
        } else {
            setMessage("Failed to get answer");
        }
    }

    @Override
    protected Control createDialogArea(Composite parent) {
        Composite area = (Composite) super.createDialogArea(parent);

        Composite container = new Composite(area, SWT.NONE);
        container.setLayoutData(new GridData(SWT.FILL, SWT.FILL, true, true));
        GridLayout layout = new GridLayout(1, false);
        layout.marginHeight = 10;
        layout.marginWidth = 10;
        container.setLayout(layout);

        if (response.hasError()) {
            createErrorSection(container);
            return area;
        }

        QAAnswer answer = response.getAnswer();
        if (answer == null) {
            Label noResultLabel = new Label(container, SWT.NONE);
            noResultLabel.setText("No answer available.");
            return area;
        }

        // Answer text section
        createAnswerSection(container, answer);

        // Code examples section
        if (answer.hasCodeExamples()) {
            createCodeExamplesSection(container, answer);
        }

        // Related topics section
        if (answer.hasRelatedTopics()) {
            createRelatedTopicsSection(container, answer);
        }

        // References section
        if (response.hasReferences()) {
            createReferencesSection(container);
        }

        return area;
    }

    private void createErrorSection(Composite parent) {
        Label errorLabel = new Label(parent, SWT.NONE);
        errorLabel.setText("Error:");
        errorLabel.setForeground(parent.getDisplay().getSystemColor(SWT.COLOR_RED));

        Text errorText = new Text(parent, SWT.READ_ONLY | SWT.MULTI | SWT.WRAP | SWT.BORDER);
        GridData errorData = new GridData(SWT.FILL, SWT.CENTER, true, false);
        errorData.heightHint = 60;
        errorText.setLayoutData(errorData);
        errorText.setText(response.getError());
        errorText.setForeground(parent.getDisplay().getSystemColor(SWT.COLOR_RED));
    }

    private void createAnswerSection(Composite parent, QAAnswer answer) {
        Label answerLabel = new Label(parent, SWT.NONE);
        answerLabel.setText("Answer:");
        answerLabel.setLayoutData(new GridData(SWT.LEFT, SWT.TOP, false, false));

        Text answerText = new Text(parent, SWT.READ_ONLY | SWT.MULTI | SWT.V_SCROLL | SWT.WRAP | SWT.BORDER);
        GridData answerData = new GridData(SWT.FILL, SWT.FILL, true, true);
        answerData.heightHint = 200;
        answerText.setLayoutData(answerData);
        answerText.setText(answer.getText() != null ? answer.getText() : "No answer text available.");
    }

    private void createCodeExamplesSection(Composite parent, QAAnswer answer) {
        Label examplesLabel = new Label(parent, SWT.NONE);
        examplesLabel.setText(String.format("Code Examples (%d):", answer.getCodeExamples().size()));
        examplesLabel.setLayoutData(new GridData(SWT.LEFT, SWT.TOP, false, false));

        Text examplesText = new Text(parent, SWT.READ_ONLY | SWT.MULTI | SWT.V_SCROLL | SWT.H_SCROLL | SWT.BORDER);
        GridData examplesData = new GridData(SWT.FILL, SWT.FILL, true, true);
        examplesData.heightHint = 150;
        examplesText.setLayoutData(examplesData);

        StringBuilder sb = new StringBuilder();
        for (int i = 0; i < answer.getCodeExamples().size(); i++) {
            CodeExample example = answer.getCodeExamples().get(i);
            if (i > 0) {
                sb.append("\n\n---\n\n");
            }
            if (example.hasDescription()) {
                sb.append("// ").append(example.getDescription()).append("\n");
            }
            sb.append("[").append(example.getLanguage()).append("]\n");
            sb.append(example.getCode());
        }
        examplesText.setText(sb.toString());
    }

    private void createRelatedTopicsSection(Composite parent, QAAnswer answer) {
        Label topicsLabel = new Label(parent, SWT.NONE);
        topicsLabel.setText("Related Topics:");
        topicsLabel.setLayoutData(new GridData(SWT.LEFT, SWT.TOP, false, false));

        Text topicsText = new Text(parent, SWT.READ_ONLY | SWT.MULTI | SWT.WRAP | SWT.BORDER);
        GridData topicsData = new GridData(SWT.FILL, SWT.CENTER, true, false);
        topicsData.heightHint = 40;
        topicsText.setLayoutData(topicsData);

        StringBuilder sb = new StringBuilder();
        for (int i = 0; i < answer.getRelatedTopics().size(); i++) {
            if (i > 0) {
                sb.append(" | ");
            }
            sb.append(answer.getRelatedTopics().get(i));
        }
        topicsText.setText(sb.toString());
        topicsText.setBackground(parent.getDisplay().getSystemColor(SWT.COLOR_WIDGET_BACKGROUND));
    }

    private void createReferencesSection(Composite parent) {
        Label refsLabel = new Label(parent, SWT.NONE);
        refsLabel.setText(String.format("Knowledge References (%d):", response.getReferences().size()));
        refsLabel.setLayoutData(new GridData(SWT.LEFT, SWT.TOP, false, false));

        Text refsText = new Text(parent, SWT.READ_ONLY | SWT.MULTI | SWT.V_SCROLL | SWT.WRAP | SWT.BORDER);
        GridData refsData = new GridData(SWT.FILL, SWT.CENTER, true, false);
        refsData.heightHint = 60;
        refsText.setLayoutData(refsData);

        StringBuilder sb = new StringBuilder();
        for (int i = 0; i < response.getReferences().size(); i++) {
            KnowledgeReference ref = response.getReferences().get(i);
            if (i > 0) {
                sb.append("\n");
            }
            sb.append(i + 1).append(". ");
            sb.append(ref.getName());
            sb.append(" [").append(ref.getCategory()).append("]");
            if (ref.getSection() != null && !ref.getSection().isEmpty()) {
                sb.append(" - ").append(ref.getSection());
            }
            sb.append(" (").append(ref.getRelevancePercent()).append(" match)");
        }
        refsText.setText(sb.toString());
        refsText.setBackground(parent.getDisplay().getSystemColor(SWT.COLOR_WIDGET_BACKGROUND));
    }

    @Override
    protected void createButtonsForButtonBar(Composite parent) {
        createButton(parent, OK, "Close", true);
    }

    @Override
    protected boolean isResizable() {
        return true;
    }
}
